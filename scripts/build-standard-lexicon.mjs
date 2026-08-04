#!/usr/bin/env node
import { createHash } from "node:crypto";
import { gzipSync } from "node:zlib";
import { readFile, mkdir, writeFile } from "node:fs/promises";
import { resolve, join } from "node:path";
import { fileURLToPath } from "node:url";

const HEADER = "geullint-standard-lexicon-v1";

export function parseLexiconRows(source) {
  if (typeof source !== "string") throw new TypeError("lexicon source must be a string");
  const lines = source.replaceAll("\r\n", "\n").split("\n");
  const start = lines[0] === "surface\tpos\tfrequency" ? 1 : 0;
  const entries = new Map();
  for (const [offset, line] of lines.slice(start).entries()) {
    if (!line.trim() || line.startsWith("#")) continue;
    const fields = line.split("\t");
    if (fields.length !== 3) throw new Error(`lexicon row ${offset + start + 1} must contain surface, pos, and frequency`);
    const [surface, pos, frequencyText] = fields.map((field) => field.trim());
    if (!surface) throw new Error(`lexicon row ${offset + start + 1} surface must be non-empty`);
    if (!pos) throw new Error(`lexicon row ${offset + start + 1} pos must be non-empty`);
    if (!/^\d+$/u.test(frequencyText)) throw new Error(`lexicon row ${offset + start + 1} frequency must be a non-negative integer`);
    const frequency = Number(frequencyText);
    if (!Number.isSafeInteger(frequency)) throw new Error(`lexicon row ${offset + start + 1} frequency exceeds safe integer range`);
    const previous = entries.get(surface);
    if (previous && previous.pos !== pos) throw new Error(`lexicon surface ${surface} has conflicting POS`);
    entries.set(surface, {
      surface,
      pos,
      frequency: (previous?.frequency ?? 0) + frequency,
    });
  }
  return [...entries.values()].sort((left, right) => left.surface < right.surface ? -1 : left.surface > right.surface ? 1 : 0);
}

export function serializeLexicon(rows) {
  if (!Array.isArray(rows)) throw new TypeError("lexicon rows must be an array");
  let previous = "";
  const lines = [HEADER];
  for (const row of rows) {
    if (!row || typeof row.surface !== "string" || !row.surface || typeof row.pos !== "string" || !row.pos || !Number.isSafeInteger(row.frequency) || row.frequency < 0) {
      throw new TypeError("lexicon rows must contain valid surface, pos, and frequency values");
    }
    if (previous && row.surface <= previous) throw new Error("lexicon rows must be sorted and unique");
    previous = row.surface;
    lines.push(`${row.surface}\t${row.pos}\t${row.frequency}`);
  }
  return Buffer.from(`${lines.join("\n")}\n`, "utf8");
}

export function buildLexicon(rows, { name = "GeulLint standard Korean lexicon", version = "1.0.0" } = {}) {
  const raw = serializeLexicon(rows);
  const gzip = gzipSync(raw, { level: 9, mtime: 0 });
  return {
    raw,
    gzip,
    manifest: {
      schemaVersion: 1,
      name,
      version,
      format: HEADER,
      entryCount: rows.length,
      rawBytes: raw.length,
      gzipBytes: gzip.length,
      sha256: digest(raw),
      gzipSha256: digest(gzip),
    },
  };
}

function digest(value) {
  return createHash("sha256").update(value).digest("hex");
}

function argument(arguments_, name) {
  const index = arguments_.indexOf(name);
  const value = index === -1 ? undefined : arguments_[index + 1];
  return value && !value.startsWith("--") ? value : undefined;
}

async function main(arguments_) {
  const input = argument(arguments_, "--input");
  const outDir = argument(arguments_, "--out-dir");
  if (!input || !outDir) throw new Error("usage: node scripts/build-standard-lexicon.mjs --input PATH --out-dir PATH [--name NAME]");
  const rows = parseLexiconRows(await readFile(resolve(input), "utf8"));
  const result = buildLexicon(rows, { name: argument(arguments_, "--name") ?? undefined });
  const output = resolve(outDir);
  await mkdir(output, { recursive: true });
  await writeFile(join(output, "standard-ko-v1.txt"), result.raw);
  await writeFile(join(output, "standard-ko-v1.txt.gz"), result.gzip);
  await writeFile(join(output, "standard-ko-v1.manifest.json"), `${JSON.stringify(result.manifest, null, 2)}\n`);
  process.stdout.write(`${JSON.stringify({ ...result.manifest, outDir: output }, null, 2)}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(`standard lexicon: ${error.message}`);
    process.exitCode = 2;
  });
}

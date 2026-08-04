#!/usr/bin/env node
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const LICENSE_PATTERN = /\S/u;

/** Find the smallest changed span using Unicode code-point indexes. */
export function changedSpan(before, after) {
  if (typeof before !== "string" || typeof after !== "string") {
    throw new TypeError("before and after must be strings");
  }
  const left = [...before];
  const right = [...after];
  let start = 0;
  while (start < left.length && start < right.length && left[start] === right[start]) start += 1;
  let leftEnd = left.length;
  let rightEnd = right.length;
  while (leftEnd > start && rightEnd > start && left[leftEnd - 1] === right[rightEnd - 1]) {
    leftEnd -= 1;
    rightEnd -= 1;
  }
  return {
    start,
    end: leftEnd,
    before: left.slice(start, leftEnd).join(""),
    after: right.slice(start, rightEnd).join(""),
  };
}

function tokenCount(value) {
  return value.trim() ? value.trim().split(/\s+/u).length : 0;
}

function isLocalEdit(span) {
  const beforeTokens = tokenCount(span.before);
  const afterTokens = tokenCount(span.after);
  if (beforeTokens === 0 && afterTokens === 0) return false;
  if (beforeTokens > 2 || afterTokens > 2) return false;
  // A true multi-word rewrite is not safe to turn into an automatic gold
  // candidate. A one-token ↔ two-token change is retained for spacing review.
  if (beforeTokens > 1 && afterTokens > 1 && !/^\s+$/u.test(span.before) && !/^\s+$/u.test(span.after)) return false;
  return true;
}

function editRatio(before, after, span) {
  const total = Math.max([...before].length, [...after].length, 1);
  return Math.max([...span.before].length, [...span.after].length) / total;
}

function validateRevision(record, index) {
  if (!record || typeof record !== "object" || Array.isArray(record)) throw new Error(`revision ${index + 1} must be an object`);
  for (const field of ["id", "documentId", "authorId", "genre", "sourceUrl", "license", "before", "after"]) {
    if (typeof record[field] !== "string" || !record[field].trim()) throw new Error(`revision ${index + 1} ${field} must be non-empty`);
  }
  if (!/^https:\/\//u.test(record.sourceUrl)) throw new Error(`revision ${record.id} sourceUrl must use HTTPS`);
  if (!LICENSE_PATTERN.test(record.license)) throw new Error(`revision ${record.id} license must be non-empty`);
  if ([...record.before].length > 20_000 || [...record.after].length > 20_000) throw new Error(`revision ${record.id} exceeds the text size limit`);
}

export function extractRevisionEdits(records, options = {}) {
  if (!Array.isArray(records)) throw new TypeError("revisions must be an array");
  const split = options.split ?? "train";
  const defaultGenre = options.genre;
  const maxChangedCodePoints = options.maxChangedCodePoints ?? 80;
  const maxEditRatio = options.maxEditRatio ?? 0.4;
  const accepted = [];
  const ids = new Set();
  for (const [index, record] of records.entries()) {
    validateRevision(record, index);
    if (ids.has(record.id)) throw new Error(`duplicate revision id: ${record.id}`);
    ids.add(record.id);
    if (record.before === record.after) continue;
    const span = changedSpan(record.before, record.after);
    if (!isLocalEdit(span)) continue;
    if (Math.max([...span.before].length, [...span.after].length) > maxChangedCodePoints) continue;
    if (editRatio(record.before, record.after, span) > maxEditRatio) continue;
    accepted.push({
      id: `revision-${record.id}`,
      text: record.before,
      expectedFixedText: record.after,
      origin: "revision",
      split: record.split ?? split,
      genre: record.genre.trim() || defaultGenre,
      documentId: record.documentId.trim(),
      authorId: record.authorId.trim(),
      provenanceId: record.id,
      sourceUrl: record.sourceUrl,
      license: record.license.trim(),
      errorFamilies: Array.isArray(record.errorFamilies) && record.errorFamilies.length > 0
        ? [...new Set(record.errorFamilies.filter((family) => typeof family === "string" && family.trim()).map((family) => family.trim()))]
        : ["revision.unclassified"],
      reviewStatus: "needs_adjudication",
      changed: { before: span.before, after: span.after },
    });
  }
  return accepted;
}

async function readJsonLines(path) {
  const contents = await readFile(path, "utf8");
  return contents.split(/\r?\n/u).filter((line) => line.trim()).map((line, index) => {
    try {
      return JSON.parse(line);
    } catch (error) {
      throw new Error(`${path}:${index + 1} is not valid JSON: ${error.message}`);
    }
  });
}

function argument(arguments_, name) {
  const index = arguments_.indexOf(name);
  const value = index === -1 ? undefined : arguments_[index + 1];
  return value && !value.startsWith("--") ? value : undefined;
}

async function main(arguments_) {
  const input = argument(arguments_, "--input");
  const output = argument(arguments_, "--out");
  if (!input || !output) throw new Error("usage: node scripts/extract-human-edits.mjs --input PATH --out PATH [--split SPLIT] [--genre GENRE]");
  const cases = extractRevisionEdits(await readJsonLines(resolve(input)), {
    split: argument(arguments_, "--split"),
    genre: argument(arguments_, "--genre"),
  });
  await writeFile(resolve(output), `${cases.map(JSON.stringify).join("\n")}\n`);
  process.stdout.write(`${JSON.stringify({ extracted: cases.length, output: resolve(output) }, null, 2)}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(`human edits: ${error.message}`);
    process.exitCode = 2;
  });
}

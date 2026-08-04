#!/usr/bin/env node
import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ALLOWED_LICENSES = new Set([
  "Apache-2.0",
  "CC-BY-2.0",
  "CC-BY-4.0",
  "CC-BY-SA-4.0",
  "GPL-3.0-or-later",
  "MIT",
]);
const HEX_SHA256 = /^[0-9a-f]{64}$/u;
const DEFAULT_MAX_BYTES = 64 * 1024 * 1024;

/**
 * Validate the checked-in source manifest before any network request is made.
 * The returned value is the same object so callers can use it as a parsed
 * manifest without losing source ordering.
 */
export function validateSourceManifest(manifest) {
  if (!manifest || typeof manifest !== "object" || Array.isArray(manifest)) {
    throw new TypeError("source manifest must be an object");
  }
  if (manifest.schemaVersion !== 1) {
    throw new Error("source manifest schemaVersion must be 1");
  }
  if (!Array.isArray(manifest.sources) || manifest.sources.length === 0) {
    throw new Error("source manifest sources must be a non-empty array");
  }
  const ids = new Set();
  const filenames = new Set();
  for (const source of manifest.sources) {
    if (!source || typeof source !== "object" || Array.isArray(source)) {
      throw new TypeError("each source manifest entry must be an object");
    }
    for (const field of ["id", "url", "recordUrl", "license", "licenseUrl", "filename"]) {
      if (typeof source[field] !== "string" || !source[field].trim()) {
        throw new Error(`source ${field} must be a non-empty string`);
      }
    }
    if (ids.has(source.id)) throw new Error(`duplicate source id: ${source.id}`);
    ids.add(source.id);
    if (!/^https:\/\//u.test(source.url) || !/^https:\/\//u.test(source.recordUrl)) {
      throw new Error(`source ${source.id} URLs must use HTTPS`);
    }
    if (!/^https:\/\//u.test(source.licenseUrl)) {
      throw new Error(`source ${source.id} licenseUrl must use HTTPS`);
    }
    if (!ALLOWED_LICENSES.has(source.license)) {
      throw new Error(`source ${source.id} license is not on the allowlist: ${source.license}`);
    }
    if (typeof source.sha256 !== "string" || !HEX_SHA256.test(source.sha256)) {
      throw new Error(`source ${source.id} sha256 must be a 64-character hex digest`);
    }
    if (typeof source.redistributable !== "boolean") {
      throw new Error(`source ${source.id} redistributable must be boolean`);
    }
    if (isAbsolute(source.filename) || source.filename.split(/[\\/]+/u).some((part) => part === "..")) {
      throw new Error(`source ${source.id} filename must be a relative file name`);
    }
    if (filenames.has(source.filename)) throw new Error(`duplicate source filename: ${source.filename}`);
    filenames.add(source.filename);
  }
  return manifest;
}

/**
 * Return sources that may be downloaded under the default policy. GPL or
 * otherwise review-only sources are never included unless the caller opts in
 * explicitly; this prevents an accidental non-redistributable bundle.
 */
export function planDownloads(manifest, options = {}) {
  validateSourceManifest(manifest);
  const accepted = options.acceptNonRedistributable === true;
  const includeNonRedistributable = options.includeNonRedistributable === true || accepted;
  if (includeNonRedistributable && !accepted) {
    throw new Error("including non-redistributable sources requires explicit acceptance");
  }
  return manifest.sources.filter(
    (source) => source.redistributable || (includeNonRedistributable && accepted),
  );
}

export async function acquireSources(
  manifest,
  {
    outDir,
    includeNonRedistributable = false,
    acceptNonRedistributable = false,
    fetchImpl = globalThis.fetch,
    maxBytes = DEFAULT_MAX_BYTES,
  } = {},
) {
  if (typeof outDir !== "string" || !outDir.trim()) throw new Error("outDir is required");
  if (typeof fetchImpl !== "function") throw new Error("a fetch implementation is required");
  if (!Number.isSafeInteger(maxBytes) || maxBytes <= 0) throw new Error("maxBytes must be positive");
  const sources = planDownloads(manifest, { includeNonRedistributable, acceptNonRedistributable });
  const output = resolve(outDir);
  await mkdir(output, { recursive: true });
  const results = [];
  for (const source of sources) {
    const response = await fetchImpl(source.url);
    if (!response?.ok) throw new Error(`source ${source.id} download failed with HTTP ${response?.status ?? "unknown"}`);
    const bytes = Buffer.from(await response.arrayBuffer());
    if (bytes.length > maxBytes) throw new Error(`source ${source.id} exceeds the ${maxBytes}-byte size limit`);
    const sha256 = digest(bytes);
    if (sha256 !== source.sha256) throw new Error(`source ${source.id} SHA-256 does not match the manifest`);
    const destination = join(output, source.filename);
    await mkdir(dirname(destination), { recursive: true });
    await writeFile(destination, bytes, { flag: "wx" });
    results.push({ ...source, path: destination, bytes: bytes.length });
  }
  const acquisition = {
    schemaVersion: 1,
    acquiredAt: new Date().toISOString(),
    sources: results,
  };
  await writeFile(join(output, "acquisition-manifest.json"), `${JSON.stringify(acquisition, null, 2)}\n`, { flag: "wx" });
  return acquisition;
}

function digest(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function parseJson(path) {
  return JSON.parse(await readFile(path, "utf8"));
}

function argument(arguments_, name) {
  const index = arguments_.indexOf(name);
  const value = index === -1 ? undefined : arguments_[index + 1];
  return value && !value.startsWith("--") ? value : undefined;
}

async function main(arguments_) {
  const manifestPath = argument(arguments_, "--manifest");
  const outDir = argument(arguments_, "--out-dir");
  if (!manifestPath || !outDir) {
    throw new Error("usage: node scripts/acquire-training-data.mjs --manifest PATH --out-dir PATH [--include-non-redistributable --accept-non-redistributable]");
  }
  const manifest = validateSourceManifest(await parseJson(resolve(manifestPath)));
  const result = await acquireSources(manifest, {
    outDir,
    includeNonRedistributable: arguments_.includes("--include-non-redistributable"),
    acceptNonRedistributable: arguments_.includes("--accept-non-redistributable"),
  });
  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(`training data: ${error.message}`);
    process.exitCode = 2;
  });
}

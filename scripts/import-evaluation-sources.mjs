#!/usr/bin/env node
import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const HEX_SHA256 = /^[0-9a-f]{64}$/u;
const HOLDOUTS = new Set(["H1", "H2"]);

function isNonblankString(value) {
  return typeof value === "string" && value.trim().length > 0;
}

function digest(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function sourceUrl(source) {
  return source.recordUrl ?? source.url;
}

export function verifyEvaluationSource(source, bytes) {
  if (!source || typeof source !== "object" || Array.isArray(source)) {
    throw new TypeError("source manifest entry must be an object");
  }
  if (!isNonblankString(source.id)) throw new Error("source id must be non-empty");
  if (source.access === "manual_authorization") {
    throw new Error(`source ${source.id} is a manual authorization source and cannot be imported`);
  }
  if (!isNonblankString(source.license)) throw new Error(`source ${source.id} license must be non-empty`);
  if (!HEX_SHA256.test(source.sha256 ?? "")) {
    throw new Error(`source ${source.id} sha256 must be a 64-character lowercase hex digest`);
  }
  if (!isNonblankString(sourceUrl(source))) throw new Error(`source ${source.id} requires a record URL`);
  if (!Buffer.isBuffer(bytes) && !(bytes instanceof Uint8Array)) {
    throw new TypeError("source bytes must be a Buffer or Uint8Array");
  }
  const sha256 = digest(bytes);
  if (sha256 !== source.sha256) {
    throw new Error(`source ${source.id} SHA-256 does not match the manifest`);
  }
  return {
    sourceId: source.id,
    sha256,
    license: source.license,
    sourceUrl: sourceUrl(source),
  };
}

function validateSplit(split) {
  if (!isNonblankString(split)) throw new Error("split must be a non-empty string");
  if (!["train", "dev", "release_holdout", ...HOLDOUTS].includes(split)) {
    throw new Error(`unknown split: ${split}`);
  }
}

export function importEvaluationSource(records, options = {}) {
  if (!Array.isArray(records) || records.length === 0) {
    throw new TypeError("source records must be a non-empty array");
  }
  const {
    source,
    bytes,
    split,
    textOrigin = "human_authored",
    annotationOrigin = "source_revision",
    annotationStatus = "unreviewed",
    forbiddenDocumentIds = [],
  } = options;
  const locked = verifyEvaluationSource(source, bytes);
  validateSplit(split);
  if (!["human_authored", "revision", "project", "synthetic"].includes(textOrigin)) {
    throw new Error(`unknown textOrigin: ${textOrigin}`);
  }
  if (annotationOrigin === "ai_blind_panel") {
    throw new Error("source importer cannot create AI blind-panel annotations; use the review merger");
  }
  if (!["source_revision", "human_independent"].includes(annotationOrigin)) {
    throw new Error(`unknown annotationOrigin: ${annotationOrigin}`);
  }
  if (!["unreviewed", "reviewed", "adjudicated", "ambiguous"].includes(annotationStatus)) {
    throw new Error(`unknown annotationStatus: ${annotationStatus}`);
  }
  const forbidden = new Set(forbiddenDocumentIds.filter(isNonblankString).map((id) => id.trim()));
  if (HOLDOUTS.has(split) && (textOrigin === "synthetic" || textOrigin === "project")) {
    throw new Error(`${textOrigin} rows are not allowed in a commercial holdout`);
  }
  const ids = new Set();
  return records.map((record) => {
    if (!record || typeof record !== "object" || Array.isArray(record)) {
      throw new TypeError("each source record must be an object");
    }
    if (!isNonblankString(record.id) || ids.has(record.id)) {
      throw new Error(`source records contain a missing or duplicate case id: ${record.id ?? ""}`);
    }
    ids.add(record.id);
    if (!isNonblankString(record.text)) throw new Error(`source case ${record.id} text must be non-empty`);
    for (const field of ["genre", "documentId", "authorId"]) {
      if (textOrigin !== "project" && !isNonblankString(record[field])) {
        throw new Error(`source case ${record.id} requires ${field}`);
      }
    }
    const documentId = record.documentId?.trim();
    if (HOLDOUTS.has(split) && forbidden.has(documentId)) {
      throw new Error(`source case ${record.id} uses a training document in a commercial holdout`);
    }
    const output = {
      ...record,
      sourceId: locked.sourceId,
      sourceSha256: locked.sha256,
      sourceUrl: locked.sourceUrl,
      license: locked.license,
      textOrigin,
      annotationOrigin,
      annotationStatus,
      split,
      holdoutId: HOLDOUTS.has(split) ? split : null,
    };
    delete output.reviewProvenance;
    return output;
  });
}

function parseJsonLines(contents, label) {
  return contents.split(/\r?\n/u).filter((line) => line.trim()).map((line, index) => {
    try {
      return JSON.parse(line);
    } catch (error) {
      throw new Error(`${label}:${index + 1} is not valid JSON: ${error.message}`);
    }
  });
}

function argument(arguments_, name, required = true) {
  const index = arguments_.indexOf(name);
  const value = index < 0 ? undefined : arguments_[index + 1];
  if (required && (!value || value.startsWith("--"))) throw new Error(`missing ${name}`);
  return value && !value.startsWith("--") ? value : undefined;
}

async function main(arguments_) {
  const input = argument(arguments_, "--input");
  const sourceManifest = argument(arguments_, "--source-manifest");
  const sourceId = argument(arguments_, "--source-id");
  const sourceBytes = argument(arguments_, "--source-bytes");
  const output = argument(arguments_, "--out");
  const manifest = JSON.parse(await readFile(resolve(sourceManifest), "utf8"));
  const source = Array.isArray(manifest.sources)
    ? manifest.sources.find((entry) => entry.id === sourceId)
    : manifest;
  if (!source) throw new Error(`source ${sourceId} was not found in the manifest`);
  const records = parseJsonLines(await readFile(resolve(input), "utf8"), input);
  const result = importEvaluationSource(records, {
    source,
    bytes: await readFile(resolve(sourceBytes)),
    split: argument(arguments_, "--split"),
    textOrigin: argument(arguments_, "--text-origin", false),
    forbiddenDocumentIds: [],
  });
  await writeFile(resolve(output), `${result.map(JSON.stringify).join("\n")}\n`, "utf8");
  process.stdout.write(`${JSON.stringify({ cases: result.length, output: resolve(output) })}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(`evaluation source: ${error.message}`);
    process.exitCode = 2;
  });
}

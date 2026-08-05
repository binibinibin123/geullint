#!/usr/bin/env node
import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const TEXT_ORIGINS = new Set(["human_authored", "revision", "project", "synthetic"]);
export const ANNOTATION_ORIGINS = new Set(["ai_blind_panel", "human_independent", "source_revision"]);
export const ANNOTATION_STATUSES = new Set(["unreviewed", "reviewed", "adjudicated", "ambiguous"]);
export const SPLITS = new Set(["train", "dev", "release_holdout", "H1", "H2"]);
const HASH_PATTERN = /^[0-9a-f]{64}$/u;
const REVIEW_STATUSES = new Set(["normal", "error", "ambiguous"]);

function isNonblankString(value) {
  return typeof value === "string" && value.trim().length > 0;
}

function requireEnum(value, allowed, field) {
  if (!allowed.has(value)) throw new Error(`${field} must be one of ${[...allowed].join(", ")}`);
}

function requireHash(value, field) {
  if (typeof value !== "string" || !HASH_PATTERN.test(value)) {
    throw new Error(`${field} must be a lowercase SHA-256 hex digest`);
  }
}

function utf8Boundaries(text) {
  const boundaries = new Set([0]);
  let offset = 0;
  for (const character of text) {
    offset += Buffer.byteLength(character, "utf8");
    boundaries.add(offset);
  }
  return boundaries;
}

function normalizeCorrection(value) {
  return value.normalize("NFKC").replace(/\s+/gu, " ").trim();
}

function normalizeDiagnostics(diagnostics) {
  return diagnostics
    .map((diagnostic) => ({
      range: { start: diagnostic.range.start, end: diagnostic.range.end },
      suggestions: [...diagnostic.suggestions].map(normalizeCorrection).sort(),
      ...(isNonblankString(diagnostic.errorFamily) ? { errorFamily: diagnostic.errorFamily.trim() } : {}),
    }))
    .sort((left, right) => `${left.range.start}:${left.range.end}`.localeCompare(`${right.range.start}:${right.range.end}`));
}

function reviewSignature(review) {
  return JSON.stringify({
    status: review.status,
    diagnostics: normalizeDiagnostics(review.diagnostics),
  });
}

function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function sha256(value) {
  return createHash("sha256").update(typeof value === "string" ? value : canonicalJson(value)).digest("hex");
}

function validateReviewProvenance(provenance, annotationOrigin, annotationStatus) {
  if (!provenance || typeof provenance !== "object" || Array.isArray(provenance)) {
    throw new Error("reviewProvenance must be an object");
  }
  if (annotationOrigin === "ai_blind_panel") {
    if (provenance.reviewerType !== "ai") throw new Error("AI annotation requires reviewerType ai");
    if (provenance.adjudicatorType !== "ai") throw new Error("AI annotation requires adjudicatorType ai");
    if (!Array.isArray(provenance.modelSnapshots) || provenance.modelSnapshots.length < 2 || provenance.modelSnapshots.some((value) => !isNonblankString(value))) {
      throw new Error("AI annotation requires at least two model snapshots");
    }
    if (Object.prototype.hasOwnProperty.call(provenance, "humanEvidence")) {
      throw new Error("AI annotation cannot carry independent human evidence");
    }
  } else if (annotationOrigin === "human_independent") {
    if (provenance.reviewerType === "ai") throw new Error("AI review cannot be labeled independent_human");
    if (provenance.reviewerType !== "human") throw new Error("human_independent annotation requires reviewerType human");
    if (!provenance.humanEvidence || !isNonblankString(provenance.humanEvidence.evidenceId) || !isNonblankString(provenance.humanEvidence.sourceUrl)) {
      throw new Error("human_independent annotation requires humanEvidence");
    }
  }
  if (annotationStatus === "adjudicated" && !isNonblankString(provenance.adjudicatorId)) {
    throw new Error("adjudicated case requires adjudicatorId");
  }
  for (const field of ["rubricSha256", "sessionSha256", "outputSha256"]) requireHash(provenance[field], `reviewProvenance.${field}`);
}

function validateDiagnostic(diagnostic, text, index) {
  if (!diagnostic || typeof diagnostic !== "object" || Array.isArray(diagnostic)) {
    throw new Error(`expectedDiagnostics[${index}] must be an object`);
  }
  const range = diagnostic.range;
  const boundaries = utf8Boundaries(text);
  if (!range || !Number.isInteger(range.start) || !Number.isInteger(range.end) || range.start >= range.end || !boundaries.has(range.start) || !boundaries.has(range.end)) {
    throw new Error(`expectedDiagnostics[${index}] range must be UTF-8 boundaries`);
  }
  if (!Array.isArray(diagnostic.suggestions) || diagnostic.suggestions.length === 0 || diagnostic.suggestions.some((suggestion) => typeof suggestion !== "string")) {
    throw new Error(`expectedDiagnostics[${index}] suggestions must be non-empty strings`);
  }
  if (diagnostic.ruleId !== undefined && !isNonblankString(diagnostic.ruleId)) throw new Error(`expectedDiagnostics[${index}] ruleId must be non-empty`);
  if (diagnostic.errorFamily !== undefined && !isNonblankString(diagnostic.errorFamily)) throw new Error(`expectedDiagnostics[${index}] errorFamily must be non-empty`);
}

export function validateCaseV2(entry) {
  if (!entry || typeof entry !== "object" || Array.isArray(entry)) throw new Error("case must be an object");
  for (const field of ["id", "text", "textOrigin", "annotationOrigin", "annotationStatus"]) {
    if (!isNonblankString(entry[field])) throw new Error(`${field} must be a non-empty string`);
  }
  requireEnum(entry.textOrigin, TEXT_ORIGINS, "textOrigin");
  requireEnum(entry.annotationOrigin, ANNOTATION_ORIGINS, "annotationOrigin");
  requireEnum(entry.annotationStatus, ANNOTATION_STATUSES, "annotationStatus");
  if (!Array.isArray(entry.expectedDiagnostics)) throw new Error("expectedDiagnostics must be an array");
  if (entry.annotationStatus === "ambiguous" && entry.expectedDiagnostics.length !== 0) {
    throw new Error("ambiguous cases cannot force expected diagnostics");
  }
  for (const [index, diagnostic] of entry.expectedDiagnostics.entries()) validateDiagnostic(diagnostic, entry.text, index);
  if (entry.textOrigin !== "project") {
    for (const field of ["genre", "split", "documentId", "authorId"]) {
      if (!isNonblankString(entry[field])) throw new Error(`non-project case requires ${field}`);
    }
    requireEnum(entry.split, SPLITS, "split");
  }
  if (entry.split === "H1" || entry.split === "H2") {
    if (entry.holdoutId !== entry.split) throw new Error("holdoutId must match the holdout split");
  } else if (entry.holdoutId !== null && entry.holdoutId !== undefined) {
    throw new Error("non-holdout case must not carry holdoutId");
  }
  if (entry.annotationOrigin === "ai_blind_panel" && entry.annotationStatus === "unreviewed") {
    throw new Error("AI panel cases must be reviewed before promotion");
  }
  validateReviewProvenance(entry.reviewProvenance, entry.annotationOrigin, entry.annotationStatus);
  return entry;
}

function validateReview(review, text, index) {
  if (!review || typeof review !== "object" || Array.isArray(review)) throw new Error(`review ${index + 1} must be an object`);
  for (const field of ["reviewerId", "reviewerType", "modelSnapshot", "rubricSha256", "sessionSha256", "outputSha256", "status"]) {
    if (!isNonblankString(review[field])) throw new Error(`review ${index + 1} ${field} must be non-empty`);
  }
  if (review.reviewerType !== "ai") throw new Error(`review ${index + 1} reviewerType must be ai`);
  requireEnum(review.status, REVIEW_STATUSES, `review ${index + 1} status`);
  for (const field of ["rubricSha256", "sessionSha256", "outputSha256"]) requireHash(review[field], `review ${index + 1} ${field}`);
  if (!Array.isArray(review.diagnostics)) throw new Error(`review ${index + 1} diagnostics must be an array`);
  if (review.status === "normal" || review.status === "ambiguous") {
    if (review.diagnostics.length !== 0) throw new Error(`review ${index + 1} ${review.status} must have no diagnostics`);
  } else {
    for (const [diagnosticIndex, diagnostic] of review.diagnostics.entries()) validateDiagnostic(diagnostic, text, diagnosticIndex);
  }
}

function provenanceFor(reviews, adjudication) {
  const all = [...reviews, ...(adjudication ? [adjudication] : [])];
  return {
    reviewerType: "ai",
    adjudicatorType: "ai",
    adjudicatorId: adjudication?.reviewerId ?? null,
    modelSnapshots: [...new Set(all.map((review) => review.modelSnapshot))].sort(),
    rubricSha256: reviews[0].rubricSha256,
    sessionSha256: sha256(reviews.map((review) => review.sessionSha256)),
    outputSha256: sha256(all.map((review) => review.outputSha256)),
  };
}

export function mergeBlindReviews(base, reviews, adjudication) {
  if (!base || typeof base !== "object" || !Array.isArray(reviews) || reviews.length < 2) {
    throw new Error("base and at least two blind reviews are required");
  }
  const reviewerIds = new Set();
  for (const [index, review] of reviews.entries()) {
    validateReview(review, base.text, index);
    if (reviewerIds.has(review.reviewerId)) throw new Error("blind reviews must use unique reviewer IDs");
    reviewerIds.add(review.reviewerId);
  }
  const signatures = new Set(reviews.map(reviewSignature));
  let selected = reviews[0];
  let annotationStatus = "reviewed";
  if (signatures.size > 1) {
    if (!adjudication) throw new Error("adjudication required for conflicting blind reviews");
    validateReview(adjudication, base.text, reviews.length);
    if (reviewerIds.has(adjudication.reviewerId)) throw new Error("adjudicator must be separate from reviewers");
    selected = adjudication;
    annotationStatus = adjudication.status === "ambiguous" ? "ambiguous" : "adjudicated";
  } else if (selected.status === "ambiguous") {
    annotationStatus = "ambiguous";
  }
  const result = {
    ...base,
    annotationOrigin: "ai_blind_panel",
    annotationStatus,
    expectedDiagnostics: annotationStatus === "ambiguous" ? [] : normalizeDiagnostics(selected.diagnostics),
    reviewProvenance: provenanceFor(reviews, signatures.size > 1 ? adjudication : undefined),
  };
  return validateCaseV2(result);
}

async function readJsonLines(path) {
  const contents = await readFile(resolve(path), "utf8");
  return contents.split(/\r?\n/u).filter((line) => line.trim()).map((line, index) => {
    try {
      return JSON.parse(line);
    } catch (error) {
      throw new Error(`${path}:${index + 1} is not valid JSON: ${error.message}`);
    }
  });
}

async function main(arguments_) {
  const input = arguments_[arguments_.indexOf("--input") + 1];
  const output = arguments_[arguments_.indexOf("--out") + 1];
  if (!input || !output) throw new Error("usage: node scripts/ai-adjudication.mjs --input PATH --out PATH");
  const cases = await readJsonLines(input);
  for (const entry of cases) validateCaseV2(entry);
  await writeFile(resolve(output), `${cases.map(JSON.stringify).join("\n")}\n`, "utf8");
  process.stdout.write(`${JSON.stringify({ cases: cases.length, output: resolve(output) })}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(`geullint: ${error.message}`);
    process.exitCode = 2;
  });
}

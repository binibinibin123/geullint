#!/usr/bin/env node
import { createHash } from "node:crypto";
import { lstat, mkdir, mkdtemp, readFile, rename, rm, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SOURCE_RECORD_URL = "https://zenodo.org/records/16908784";
const LICENSE = "GPL-3.0-or-later";
const CORPUS_NAME = "KoLLA v2 manually curated GeulLint gold corpus";
const CORPUS_FILE = "kolla-v2-curated-gold.jsonl";
const MANIFEST_FILE = "kolla-v2-curated-gold.manifest.json";
const PROVENANCE_FILE = "kolla-v2-curated-gold.provenance.json";
const PROVENANCE_SHA256_FILE = "kolla-v2-curated-gold.provenance.sha256";

export function curateGoldCases(reviewCases, mapping, options = {}) {
  if (mapping?.schemaVersion !== 1 || !Array.isArray(mapping.cases)) {
    throw new Error("mapping schemaVersion must be 1 and cases must be an array");
  }
  if (mapping.cases.length === 0) {
    throw new Error("mapping must contain at least one reviewed case");
  }

  const reviewsById = new Map();
  for (const review of reviewCases) {
    if (!isNonblankString(review?.id)) {
      throw new Error("review queue contains an invalid review ID");
    }
    if (!isNonblankString(review?.text)) {
      throw new Error(`review queue ${review.id} contains invalid review text`);
    }
    if (reviewsById.has(review.id)) {
      throw new Error("review queue contains an invalid or duplicate case ID");
    }
    reviewsById.set(review.id, review);
  }
  const selectedReviewIds = new Set();
  return mapping.cases.map((selection) => {
    const reviewId = selection?.reviewId;
    if (!isNonblankString(reviewId)) {
      throw new Error("mapping contains an invalid review ID");
    }
    const review = reviewsById.get(reviewId);
    if (!review || selectedReviewIds.has(reviewId)) {
      throw new Error(`mapping references an unknown or duplicate review ID: ${reviewId}`);
    }
    selectedReviewIds.add(reviewId);
    if (!Array.isArray(selection.expectedDiagnostics) || selection.expectedDiagnostics.length === 0) {
      throw new Error(`mapping ${reviewId} must contain exact expectedDiagnostics`);
    }
    const boundaries = utf8ByteBoundaries(review.text);
    for (const diagnostic of selection.expectedDiagnostics) {
      validateExpectedDiagnostic(diagnostic, boundaries, reviewId);
    }
    if (options.requireIndependentReview) {
      validateIndependentReview(selection, boundaries, reviewId);
    }
    return {
      id: reviewId,
      text: review.text,
      sourceKind: "plain_text",
      expectedDiagnostics: selection.expectedDiagnostics,
    };
  });
}

function validateIndependentReview(selection, boundaries, reviewId) {
  const reviews = selection.independentReviews;
  if (!Array.isArray(reviews) || reviews.length < 2) {
    throw new Error(`mapping ${reviewId} must contain at least two independent reviewers`);
  }
  const reviewers = new Set();
  for (const review of reviews) {
    if (!isNonblankString(review?.reviewer) || reviewers.has(review.reviewer)) {
      throw new Error(`mapping ${reviewId} must contain unique independent reviewer IDs`);
    }
    reviewers.add(review.reviewer);
    if (!Array.isArray(review.expectedDiagnostics) || review.expectedDiagnostics.length === 0) {
      throw new Error(`independent reviewer ${review.reviewer} must contain exact expectedDiagnostics`);
    }
    for (const diagnostic of review.expectedDiagnostics) {
      validateExpectedDiagnostic(diagnostic, boundaries, reviewId);
    }
  }
  if (!isNonblankString(selection.adjudicatedBy) || reviewers.has(selection.adjudicatedBy)) {
    throw new Error(`mapping ${reviewId} must contain a separate adjudicator`);
  }
}

function validateExpectedDiagnostic(diagnostic, boundaries, reviewId) {
  const range = diagnostic?.range;
  if (
    !isNonblankString(diagnostic?.ruleId)
    || !range
    || !Number.isInteger(range.start)
    || !Number.isInteger(range.end)
    || range.start >= range.end
    || !boundaries.has(range.start)
    || !boundaries.has(range.end)
    || !Array.isArray(diagnostic.suggestions)
    || diagnostic.suggestions.length === 0
    || diagnostic.suggestions.some((suggestion) => !isNonblankString(suggestion))
  ) {
    throw new Error(`mapping ${reviewId} has an invalid exact diagnostic`);
  }
}

function utf8ByteBoundaries(text) {
  const boundaries = new Set([0]);
  let offset = 0;
  for (const character of text) {
    offset += Buffer.byteLength(character, "utf8");
    boundaries.add(offset);
  }
  return boundaries;
}

async function main(cliArguments) {
  const reviewQueuePath = requiredArgument(cliArguments, "--review-queue");
  const mappingPath = requiredArgument(cliArguments, "--mapping");
  const outputDirectory = requiredArgument(cliArguments, "--out-dir");
  if (!reviewQueuePath || !mappingPath || !outputDirectory) {
    throw new Error(
      "Usage: node scripts/curate-kolla-v2-gold.mjs --review-queue PATH --mapping PATH --out-dir PATH",
    );
  }
  const requireIndependentReview = cliArguments.includes("--require-independent-review");
  if (cliArguments.includes("--verify")) {
    await verifyCurationBundle(reviewQueuePath, mappingPath, resolve(outputDirectory), {
      requireIndependentReview,
    });
    console.log(JSON.stringify({ verified: true }));
    return;
  }

  const [reviewQueueBytes, mappingBytes] = await Promise.all([
    readFile(reviewQueuePath),
    readFile(mappingPath),
  ]);
  const reviewCases = parseJsonLines(reviewQueueBytes.toString("utf8"), reviewQueuePath);
  const mapping = JSON.parse(mappingBytes.toString("utf8"));
  const goldCases = curateGoldCases(reviewCases, mapping, { requireIndependentReview });
  const goldCorpus = `${goldCases.map(JSON.stringify).join("\n")}\n`;
  const corpusSha256 = digest("sha256", goldCorpus);
  const output = resolve(outputDirectory);
  const manifest = `${JSON.stringify(
    {
      schemaVersion: 1,
      name: CORPUS_NAME,
      license: LICENSE,
      sourceUrl: SOURCE_RECORD_URL,
      corpusPath: CORPUS_FILE,
      sha256: corpusSha256,
    },
    null,
    2,
  )}\n`;
  const provenance = `${JSON.stringify(
    {
      schemaVersion: 1,
      reviewQueueSha256: digest("sha256", reviewQueueBytes),
      mappingSha256: digest("sha256", mappingBytes),
      corpusSha256,
      manifestSha256: digest("sha256", manifest),
      independentReviewRequired: requireIndependentReview,
      cases: goldCases.length,
    },
    null,
    2,
  )}\n`;
  await publishCurationBundle(output, [
    [CORPUS_FILE, goldCorpus],
    [MANIFEST_FILE, manifest],
    [PROVENANCE_FILE, provenance],
    [
      PROVENANCE_SHA256_FILE,
      `${digest("sha256", provenance)}  ${PROVENANCE_FILE}\n`,
    ],
  ]);
  console.log(
    JSON.stringify({
      cases: goldCases.length,
      corpusManifest: `${output}/${MANIFEST_FILE}`,
      curationProvenance: `${output}/${PROVENANCE_FILE}`,
      curationProvenanceSha256: `${output}/${PROVENANCE_SHA256_FILE}`,
    }),
  );
}

async function verifyCurationBundle(reviewQueuePath, mappingPath, output, options = {}) {
  const [reviewQueueBytes, mappingBytes, corpusBytes, manifestBytes, provenanceBytes, sidecarBytes] =
    await Promise.all([
      readFile(reviewQueuePath),
      readFile(mappingPath),
      readFile(`${output}/${CORPUS_FILE}`),
      readFile(`${output}/${MANIFEST_FILE}`),
      readFile(`${output}/${PROVENANCE_FILE}`),
      readFile(`${output}/${PROVENANCE_SHA256_FILE}`),
    ]);
  const sidecar = sidecarBytes.toString("utf8").trimEnd();
  const expectedSidecar = `${digest("sha256", provenanceBytes)}  ${PROVENANCE_FILE}`;
  if (sidecar !== expectedSidecar) {
    throw new Error("curation provenance SHA-256 does not match its sidecar");
  }
  const provenance = JSON.parse(provenanceBytes.toString("utf8"));
  if (provenance?.schemaVersion !== 1) {
    throw new Error("curation provenance schemaVersion must be 1");
  }
  if (options.requireIndependentReview && provenance.independentReviewRequired !== true) {
    throw new Error("curation provenance does not require independent review");
  }
  const manifest = JSON.parse(manifestBytes.toString("utf8"));
  if (manifest?.schemaVersion !== 1) {
    throw new Error("curation manifest schemaVersion must be 1");
  }
  if (manifest.corpusPath !== CORPUS_FILE) {
    throw new Error("curation manifest corpusPath does not match the curated corpus");
  }
  for (const [field, expected] of [
    ["name", CORPUS_NAME],
    ["license", LICENSE],
    ["sourceUrl", SOURCE_RECORD_URL],
  ]) {
    if (manifest[field] !== expected) {
      throw new Error(`curation manifest ${field} does not match`);
    }
  }
  const corpusSha256 = digest("sha256", corpusBytes);
  if (
    manifest.sha256 !== corpusSha256
    || manifest.sha256 !== provenance.corpusSha256
  ) {
    throw new Error("curation manifest corpus SHA-256 does not match");
  }
  if (provenance.manifestSha256 !== digest("sha256", manifestBytes)) {
    throw new Error("curation provenance manifest SHA-256 does not match");
  }
  for (const [label, field, actual] of [
    ["review queue", "reviewQueueSha256", digest("sha256", reviewQueueBytes)],
    ["mapping", "mappingSha256", digest("sha256", mappingBytes)],
    ["corpus", "corpusSha256", corpusSha256],
  ]) {
    if (provenance[field] !== actual) {
      throw new Error(`curation provenance ${label} SHA-256 does not match`);
    }
  }
}

async function publishCurationBundle(output, files) {
  await mkdir(dirname(output), { recursive: true });
  const temporaryOutput = await mkdtemp(`${output}.tmp-`);
  try {
    for (const [name, contents] of files) {
      await writeFile(join(temporaryOutput, name), contents, { flag: "wx" });
    }
    await requireMissingPath(output);
    await rename(temporaryOutput, output);
  } catch (error) {
    await rm(temporaryOutput, { recursive: true, force: true });
    throw error;
  }
}

async function requireMissingPath(path) {
  try {
    await lstat(path);
  } catch (error) {
    if (error?.code === "ENOENT") {
      return;
    }
    throw error;
  }
  throw new Error(`output directory already exists: ${path}`);
}

function isNonblankString(value) {
  return typeof value === "string" && value.trim().length > 0;
}

function requiredArgument(arguments_, name) {
  const index = arguments_.indexOf(name);
  const value = index === -1 ? undefined : arguments_[index + 1];
  return value && !value.startsWith("--") ? value : undefined;
}

function parseJsonLines(source, path) {
  return source
    .split(/\r?\n/u)
    .filter((line) => line.trim())
    .map((line, index) => {
      try {
        return JSON.parse(line);
      } catch {
        throw new Error(`${path} line ${index + 1} is not valid JSON Lines data`);
      }
    });
}

function digest(algorithm, value) {
  return createHash(algorithm).update(value).digest("hex");
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(`geullint: ${error.message}`);
    process.exitCode = 2;
  });
}

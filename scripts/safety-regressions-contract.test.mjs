import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import test from "node:test";

import { validateSafetyCorpus } from "./validate-safety-corpus.mjs";

const corpusPath = "corpus/safety-regressions-v1.jsonl";
const policyPath = "corpus/safety-regressions-v1.policy.json";
const manifestPath = "corpus/safety-regressions-v1.manifest.json";

test("committed safety regressions satisfy the release structure policy", () => {
  const jsonl = readFileSync(corpusPath, "utf8");
  const policy = JSON.parse(readFileSync(policyPath, "utf8"));
  const result = validateSafetyCorpus({ jsonl, policy });

  assert.equal(result.valid, true, result.errors.join("\n"));
  assert.deepEqual(result.summary, {
    cases: 144,
    errorCases: 72,
    normalCases: 72,
    genres: 8,
    sourceKinds: 6,
    profiles: 3,
    normalizedDuplicateCount: 0,
  });
});

test("safety regression manifest fixes the exact project-owned bytes", () => {
  const corpus = readFileSync(corpusPath);
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));

  assert.equal(manifest.schemaVersion, 1);
  assert.equal(manifest.name, "GeulLint Korean safety regressions v1");
  assert.equal(manifest.license, "MIT");
  assert.equal(manifest.sourceUrl, "https://github.com/binibinibin123/geullint");
  assert.equal(manifest.corpusPath, "safety-regressions-v1.jsonl");
  assert.equal(manifest.sha256, createHash("sha256").update(corpus).digest("hex"));
});

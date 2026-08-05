import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import test from "node:test";

import { evaluateReviewQuality } from "./evaluate-review-quality.mjs";

const hash = (value) => createHash("sha256").update(value).digest("hex");
const review = (caseId, reviewerId, status = "normal", diagnostics = []) => ({
  caseId,
  reviewerId,
  reviewerType: "ai",
  modelSnapshot: `model-${reviewerId}`,
  rubricSha256: hash("a"),
  sessionSha256: hash(reviewerId),
  outputSha256: hash("b"),
  status,
  diagnostics,
});

const gate = {
  minCases: 2,
  minReviewers: 2,
  minAgreementRate: 0.5,
  maxAdjudicationRate: 0.5,
  maxAuditDisagreementRate: 0,
  maxMissingProvenanceHashes: 0,
};

test("reports agreement, adjudication, audit disagreement, and provenance metrics", () => {
  const result = evaluateReviewQuality(
    [review("agree", "a"), review("agree", "b"), review("conflict", "a"), review("conflict", "b", "error")],
    [review("conflict", "adjudicator")],
    gate,
  );
  assert.equal(result.metrics.cases, 2);
  assert.equal(result.metrics.unanimousCases, 1);
  assert.equal(result.metrics.conflictCases, 1);
  assert.equal(result.metrics.adjudicatedCases, 1);
  assert.equal(result.metrics.reviewers, 2);
  assert.equal(result.metrics.agreementRate, 0.5);
  assert.equal(result.metrics.adjudicationRate, 0.5);
  assert.equal(result.metrics.auditDisagreementRate, 0);
  assert.equal(result.metrics.missingProvenanceHashes, 0);
  assert.equal(result.passed, true);
});

test("fails review quality when conflict is unresolved or hashes are missing", () => {
  const broken = review("broken", "a");
  delete broken.outputSha256;
  const result = evaluateReviewQuality(
    [broken, review("broken", "b", "error")],
    [],
    { ...gate, minAgreementRate: 1, maxAdjudicationRate: 0 },
  );
  assert.equal(result.passed, false);
  assert.ok(result.failures.some((failure) => failure.metric === "missingAdjudication"));
  assert.ok(result.failures.some((failure) => failure.metric === "missingProvenanceHashes"));
  assert.ok(result.failures.some((failure) => failure.metric === "agreementRate"));
});

test("does not silently count a human packet in the AI panel", () => {
  const human = review("case", "human");
  human.reviewerType = "human";
  const result = evaluateReviewQuality([human, review("case", "ai")], [], gate);
  assert.equal(result.passed, false);
  assert.ok(result.failures.some((failure) => failure.metric === "nonAiReviewers"));
});

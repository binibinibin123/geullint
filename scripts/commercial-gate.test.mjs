import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

test("commercial-near gate keeps independent data and safety thresholds", async () => {
  const gate = JSON.parse(
    await readFile("corpus/gates/commercial-near-v1.json", "utf8"),
  );
  assert.equal(gate.schemaVersion, 1);
  assert.ok(gate.minMicroPrecision >= 0.98);
  assert.ok(gate.minRecall >= 0.85);
  assert.ok(gate.minTop1CorrectionAccuracy >= 0.8);
  assert.ok(gate.minTop5CorrectionAccuracy >= gate.minTop1CorrectionAccuracy);
  assert.ok(gate.minRulePrecisionWilsonLower95 >= 0.995);
  assert.ok(gate.dataset.minCases >= 20000);
  assert.ok(gate.dataset.minHumanEditCases >= 5000);
  assert.ok(gate.dataset.minNormalCases >= 10000);
  assert.ok(gate.dataset.minGenres >= 8);
  assert.ok(gate.dataset.minAuthors >= 1);
  assert.ok(gate.dataset.minHoldoutCases >= 1);
  assert.deepEqual(gate.dataset.requiredHoldoutIds, ["H1", "H2"]);
  assert.equal(gate.dataset.requireReleaseHoldout, true);
  assert.equal(gate.dataset.requireIndependentHuman, true);
  assert.equal(gate.dataset.rejectSynthetic, true);
});

test("model-adjudicated gate makes the AI-versus-human boundary explicit", async () => {
  const gate = JSON.parse(
    await readFile("corpus/gates/model-adjudicated-v1.json", "utf8"),
  );
  assert.equal(gate.schemaVersion, 1);
  assert.equal(gate.policy.aiReviewIsNotHumanEvidence, true);
  assert.equal(gate.policy.requiresIndependentHumanHoldoutForCommercialClaims, true);
  assert.ok(gate.reviewQuality.minReviewers >= 2);
});

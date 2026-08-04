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
  assert.ok(gate.minRulePrecisionWilsonLower95 >= 0.995);
  assert.ok(gate.dataset.minCases >= 20000);
  assert.ok(gate.dataset.minHumanEditCases >= 5000);
  assert.ok(gate.dataset.minNormalCases >= 10000);
  assert.ok(gate.dataset.minGenres >= 8);
  assert.equal(gate.dataset.requireReleaseHoldout, true);
  assert.equal(gate.dataset.requireIndependentHuman, true);
  assert.equal(gate.dataset.rejectSynthetic, true);
});

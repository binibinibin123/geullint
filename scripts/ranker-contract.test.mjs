import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

test("GeulRank-small manifest keeps one portable feature order and int8 bounds", async () => {
  const manifest = JSON.parse(await readFile("models/geulrank-small/manifest.json", "utf8"));
  assert.equal(manifest.schemaVersion, 1);
  assert.equal(manifest.format, "geulrank-linear-int8-v1");
  assert.deepEqual(manifest.features, ["bias", "edit_distance", "phonology_distance", "log_frequency", "base_score"]);
  for (const value of Object.values(manifest.weights)) {
    assert.ok(Number.isInteger(value) && value >= -127 && value <= 127);
  }
  assert.equal(manifest.training.documentDisjoint, true);
  assert.equal(manifest.training.releaseHoldoutRequired, true);
});

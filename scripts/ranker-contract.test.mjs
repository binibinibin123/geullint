import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createHash } from "node:crypto";
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

test("checks the experimental learned INT8 context ranker provenance", async () => {
  const root = "models/geulrank-small/context-ranker";
  const manifest = JSON.parse(await readFile(`${root}/manifest.json`, "utf8"));
  const modelCard = await readFile(`${root}/MODEL_CARD.md`, "utf8");
  const jsonArtifact = await readFile(`${root}/${manifest.jsonArtifact}`);
  const onnxArtifact = await readFile(`${root}/${manifest.onnxArtifact}`);
  assert.equal(manifest.schemaVersion, 1);
  assert.equal(manifest.format, "geulrank-context-linear-int8-v1");
  assert.equal(manifest.onnx, true);
  assert.equal(manifest.featureDim, 260);
  assert.equal(manifest.training.releaseHoldoutExcluded, true);
  assert.match(manifest.training.source, /training-only/u);
  assert.match(modelCard, /Review-only/u);
  assert.match(modelCard, /independently adjudicated/u);
  assert.equal(createHash("sha256").update(jsonArtifact).digest("hex"), manifest.jsonSha256);
  assert.equal(createHash("sha256").update(onnxArtifact).digest("hex"), manifest.onnxSha256);
  const weights = JSON.parse(jsonArtifact.toString("utf8")).weights;
  assert.equal(weights.length, manifest.featureDim);
  assert.ok(weights.every((value) => Number.isInteger(value) && value >= -127 && value <= 127));
  assert.ok(onnxArtifact.length > 0);
});

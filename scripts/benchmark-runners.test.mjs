import assert from "node:assert/strict";
import test from "node:test";
import { cargoFeatureArguments } from "./benchmark-native.mjs";
import { measureWasmFixture } from "./benchmark-wasm.mjs";

test("maps each native benchmark mode to an explicit Cargo feature set", () => {
  assert.deepEqual(cargoFeatureArguments("compact"), ["--no-default-features"]);
  assert.deepEqual(cargoFeatureArguments("source"), []);
  assert.deepEqual(cargoFeatureArguments("morphology"), ["--features", "morphology"]);
  assert.throws(() => cargoFeatureArguments("unknown"), /compact, source, or morphology/u);
});

test("measures the browser JSON boundary after warmup with an injected clock", () => {
  const calls = [];
  const times = [0, 2, 2, 5];
  const fixture = {
    id: "plain-1kb",
    sourceKind: "plain_text",
    byteLength: 1_024,
    sha256: "a".repeat(64),
    text: "검사 문장",
  };
  const lintJson = (request) => {
    calls.push(JSON.parse(request));
    return JSON.stringify({ diagnostics: [{ ruleId: "sample" }] });
  };

  const result = measureWasmFixture(lintJson, fixture, {
    warmup: 1,
    iterations: 2,
    now: () => times.shift(),
  });

  assert.equal(calls.length, 3);
  assert.deepEqual(calls[0], {
    text: "검사 문장",
    sourceKind: "plain_text",
    config: { profile: "default" },
    includeReviewFixes: false,
  });
  assert.equal(result.diagnostics, 1);
  assert.deepEqual(result.samplesMs, [2, 3]);
  assert.equal(result.summary.p50Ms, 2.5);
});

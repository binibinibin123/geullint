import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

test("keeps the release engine compact while retaining opt-in morphology", () => {
  const manifest = readFileSync("crates/geullint-core/Cargo.toml", "utf8");

  assert.match(manifest, /^default = \[\]$/mu);
  assert.match(manifest, /^morphology = \["dep:lindera"\]$/mu);
  assert.doesNotMatch(manifest, /source-parsing/u);
  assert.doesNotMatch(manifest, /^default = \[[^\]]*"morphology"/mu);
});

test("tracks explicit raw and gzip budgets for browser release artifacts", () => {
  const budget = JSON.parse(readFileSync("artifact-budgets.json", "utf8"));

  assert.equal(budget.schemaVersion, 1);
  assert.deepEqual(
    budget.artifacts.map(({ path }) => path),
    [
      "apps/playground/pkg/geullint_wasm_bg.wasm",
      "apps/playground/pkg/geullint_wasm.js",
    ],
  );
  for (const artifact of budget.artifacts) {
    assert.ok(artifact.maxRawBytes > 0);
    assert.ok(artifact.maxGzipBytes > 0);
    assert.ok(artifact.maxGzipBytes < artifact.maxRawBytes);
  }
});

test("documents reproducible measurements without unsupported comparisons", () => {
  const performance = readFileSync("docs/performance.md", "utf8");

  assert.match(performance, /fface5df1efec24e8ca5270710e8b86d7bbfe9c2/u);
  assert.match(performance, /warmup 3회.*20회/u);
  assert.match(performance, /621,434 B/u);
  assert.match(performance, /212,557 B/u);
  assert.match(performance, /benchmark-native\.mjs/u);
  assert.match(performance, /benchmark-wasm\.mjs/u);
  assert.doesNotMatch(performance, /Tree-sitter|source-parsing/u);
  assert.doesNotMatch(performance, /Harper|동급|더 빠르/u);
});

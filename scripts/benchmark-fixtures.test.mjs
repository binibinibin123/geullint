import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  BENCHMARK_SIZES,
  buildBenchmarkFixtures,
  writeBenchmarkFixtures,
} from "./benchmark-fixtures.mjs";

const encoder = new TextEncoder();

test("builds deterministic UTF-8 fixtures at the documented byte sizes", () => {
  const first = buildBenchmarkFixtures();
  const second = buildBenchmarkFixtures();

  assert.deepEqual(first, second);
  assert.equal(first.length, 9);

  for (const fixture of first) {
    assert.equal(
      encoder.encode(fixture.text).byteLength,
      BENCHMARK_SIZES[fixture.size],
      fixture.id,
    );
    assert.match(fixture.sha256, /^[a-f0-9]{64}$/u, fixture.id);
  }
});

test("covers plain text, Markdown, and TypeScript with varied generated records", () => {
  const fixtures = buildBenchmarkFixtures();
  const sourceKinds = new Set(fixtures.map(({ sourceKind }) => sourceKind));
  const largeFixtures = fixtures.filter(({ size }) => size === "1mb");

  assert.deepEqual(sourceKinds, new Set(["plain_text", "markdown", "typescript"]));
  assert.equal(largeFixtures.length, 3);

  for (const fixture of largeFixtures) {
    const recordIds = new Set(fixture.text.match(/기록\s+\d{6}/gu) ?? []);
    assert.ok(recordIds.size >= 1_000, `${fixture.id} has varied records`);
  }

  const markdown = fixtures.find(({ id }) => id === "markdown-1kb").text;
  const typescript = fixtures.find(({ id }) => id === "typescript-1kb").text;
  assert.match(markdown, /^### /mu);
  assert.match(markdown, /`token_/u);
  assert.match(typescript, /^const record\d+ =/mu);
  assert.match(typescript, /\/\//u);
  assert.match(typescript, /\/\*/u);
});

test("writes a stable manifest and source-kind-specific fixture files", (t) => {
  const directory = mkdtempSync(join(tmpdir(), "geullint-benchmark-fixtures-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));

  const manifest = writeBenchmarkFixtures(directory);
  const persisted = JSON.parse(readFileSync(join(directory, "manifest.json"), "utf8"));

  assert.deepEqual(persisted, manifest);
  assert.equal(manifest.schemaVersion, 1);
  assert.equal(manifest.fixtures.length, 9);
  assert.deepEqual(
    new Set(manifest.fixtures.map(({ path }) => path.split(".").at(-1))),
    new Set(["txt", "md", "ts"]),
  );
  for (const fixture of manifest.fixtures) {
    assert.equal(readFileSync(join(directory, fixture.path)).byteLength, fixture.byteLength);
  }
});

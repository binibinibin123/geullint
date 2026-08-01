import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { gzipSync } from "node:zlib";
import { checkArtifactBudgets } from "./artifact-budgets.mjs";

function temporaryDirectory(t) {
  const directory = mkdtempSync(join(tmpdir(), "geullint-artifact-budget-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  return directory;
}

test("reports deterministic raw and gzip sizes within budget", (t) => {
  const directory = temporaryDirectory(t);
  const artifact = Buffer.from("GeulLint offline artifact\n".repeat(20), "utf8");
  writeFileSync(join(directory, "engine.wasm"), artifact);
  const gzipBytes = gzipSync(artifact, { level: 9 }).byteLength;

  const report = checkArtifactBudgets({
    root: directory,
    artifacts: [{
      name: "browser engine",
      path: "engine.wasm",
      maxRawBytes: artifact.byteLength,
      maxGzipBytes: gzipBytes,
    }],
  });

  assert.equal(report.passed, true);
  assert.deepEqual(report.artifacts[0], {
    name: "browser engine",
    path: "engine.wasm",
    rawBytes: artifact.byteLength,
    gzipBytes,
    maxRawBytes: artifact.byteLength,
    maxGzipBytes: gzipBytes,
    passed: true,
  });
});

test("fails when either compressed or uncompressed output exceeds its budget", (t) => {
  const directory = temporaryDirectory(t);
  writeFileSync(join(directory, "engine.wasm"), Buffer.alloc(1_024, 7));

  const report = checkArtifactBudgets({
    root: directory,
    artifacts: [{
      name: "browser engine",
      path: "engine.wasm",
      maxRawBytes: 1_023,
      maxGzipBytes: 1,
    }],
  });

  assert.equal(report.passed, false);
  assert.equal(report.artifacts[0].passed, false);
  assert.equal(report.artifacts[0].rawBytes, 1_024);
  assert.ok(report.artifacts[0].gzipBytes > 1);
});

test("rejects malformed budgets before reading artifacts", () => {
  assert.throws(
    () => checkArtifactBudgets({ root: ".", artifacts: [] }),
    /at least one artifact/u,
  );
  assert.throws(
    () => checkArtifactBudgets({
      root: ".",
      artifacts: [{ name: "bad", path: "bad.wasm", maxRawBytes: 0, maxGzipBytes: 1 }],
    }),
    /positive integer/u,
  );
});

import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { gzipSync } from "node:zlib";

import { checkWasmSize } from "./check-wasm-size.mjs";
import { assertRuntimeParity, loadSourceFixture } from "./wasm-runtime-parity.mjs";

const buildScript = readFileSync("scripts/build-playground.mjs", "utf8");
const budget = JSON.parse(readFileSync("scripts/wasm-size-budget.json", "utf8"));

test("builds the playground WASM from the locked dependency graph", () => {
  assert.match(buildScript, /"--locked"/u);
  assert.match(buildScript, /"--target", "web"/u);
  assert.match(buildScript, /CARGO_PROFILE_RELEASE_OPT_LEVEL: "z"/u);
  assert.match(buildScript, /CARGO_PROFILE_RELEASE_LTO: "fat"/u);
});

test("commits a strict raw and gzip WASM size budget", () => {
  assert.deepEqual(budget, {
    version: 1,
    artifact: "apps/playground/pkg/geullint_wasm_bg.wasm",
    maxRawBytes: 650000,
    maxGzipBytes: 220000,
  });
});

test("size checker reports exact raw and gzip sizes and rejects an oversized artifact", () => {
  const directory = mkdtempSync(join(tmpdir(), "geullint-wasm-size-"));
  try {
    const wasmPath = join(directory, "fixture.wasm");
    const budgetPath = join(directory, "budget.json");
    const bytes = Buffer.from("compact wasm fixture");
    writeFileSync(wasmPath, bytes);
    writeFileSync(
      budgetPath,
      JSON.stringify({
        version: 1,
        artifact: "fixture.wasm",
        maxRawBytes: bytes.length,
        maxGzipBytes: gzipSync(bytes, { level: 9 }).length,
      }),
    );

    assert.deepEqual(checkWasmSize({ wasmPath, budgetPath }), {
      rawBytes: bytes.length,
      gzipBytes: gzipSync(bytes, { level: 9 }).length,
    });

    writeFileSync(
      budgetPath,
      JSON.stringify({
        version: 1,
        artifact: "fixture.wasm",
        maxRawBytes: bytes.length - 1,
        maxGzipBytes: 1,
      }),
    );
    assert.throws(
      () => checkWasmSize({ wasmPath, budgetPath }),
      /WASM size budget exceeded.*raw/u,
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("size checker fails clearly when the generated package is missing", () => {
  assert.throws(
    () =>
      checkWasmSize({
        wasmPath: "target/definitely-missing/geullint_wasm_bg.wasm",
        budgetPath: "scripts/wasm-size-budget.json",
      }),
    /WASM package is missing.*build-playground/u,
  );
});

test("runtime parity compares the ordered public response contract", () => {
  const fixture = loadSourceFixture();
  const selected = fixture.cases.filter(({ id }) =>
    [
      "python-string-comment-marker",
      "javascript-template-interpolation-comment",
      "rust-lifetime-and-comment",
      "markdown-prose-and-code",
    ].includes(id),
  );
  assert.equal(selected.length, 4);

  for (const sourceCase of selected) {
    const response = {
      diagnostics: sourceCase.diagnostics.map((diagnostic) => ({
        ruleId: diagnostic.ruleId,
        range: { start: diagnostic.start, end: diagnostic.end },
        original: diagnostic.original,
        suggestions: [diagnostic.suggestion],
        safeFix: diagnostic.safeFix,
      })),
      fixedText: sourceCase.fixedText,
    };
    assert.doesNotThrow(() => assertRuntimeParity(sourceCase, response));
  }

  assert.throws(
    () =>
      assertRuntimeParity(selected[0], {
        diagnostics: [],
        fixedText: selected[0].text,
      }),
    /python-string-comment-marker.*diagnostics/u,
  );
});

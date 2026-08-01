import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const workspace = resolve(import.meta.dirname, "..");
const defaultFixturePath = resolve(
  workspace,
  "crates",
  "geullint-core",
  "tests",
  "fixtures",
  "source-parity.json",
);
const packageDirectory = resolve(workspace, "apps", "playground", "pkg");
const defaultJavaScriptPath = resolve(packageDirectory, "geullint_wasm.js");
const defaultWasmPath = resolve(packageDirectory, "geullint_wasm_bg.wasm");

export function loadSourceFixture(fixturePath = defaultFixturePath) {
  const fixture = JSON.parse(readFileSync(fixturePath, "utf8"));
  assert.equal(fixture.version, 1, "source parity fixture version must be 1");
  assert.ok(Array.isArray(fixture.cases) && fixture.cases.length > 0);
  return fixture;
}

export function assertRuntimeParity(sourceCase, response) {
  const actualDiagnostics = response.diagnostics.map((diagnostic) => ({
    ruleId: diagnostic.ruleId,
    start: diagnostic.range.start,
    end: diagnostic.range.end,
    original: diagnostic.original,
    suggestion: diagnostic.suggestions[0],
    safeFix: diagnostic.safeFix,
  }));
  assert.deepEqual(
    actualDiagnostics,
    sourceCase.diagnostics,
    `${sourceCase.id}: diagnostics differ`,
  );
  assert.equal(response.fixedText, sourceCase.fixedText, `${sourceCase.id}: fixed text differs`);
}

export async function checkWasmRuntimeParity({
  fixturePath = defaultFixturePath,
  javaScriptPath = defaultJavaScriptPath,
  wasmPath = defaultWasmPath,
} = {}) {
  for (const path of [javaScriptPath, wasmPath]) {
    if (!existsSync(path)) {
      throw new Error(
        `WASM package is missing: ${path}. Run node scripts/build-playground.mjs first.`,
      );
    }
  }

  const bindings = await import(pathToFileURL(javaScriptPath).href);
  await bindings.default({ module_or_path: readFileSync(wasmPath) });
  const fixture = loadSourceFixture(fixturePath);

  for (const sourceCase of fixture.cases) {
    const response = JSON.parse(
      bindings.lint_json(
        JSON.stringify({
          text: sourceCase.text,
          sourceKind: sourceCase.sourceKind,
          config: { profile: sourceCase.profile },
        }),
      ),
    );
    assertRuntimeParity(sourceCase, response);
  }

  return fixture.cases.length;
}

function isMainModule() {
  return process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url);
}

if (isMainModule()) {
  try {
    const caseCount = await checkWasmRuntimeParity();
    process.stdout.write(`WASM runtime parity OK: ${caseCount} cases\n`);
  } catch (error) {
    process.stderr.write(`${error.stack ?? error.message}\n`);
    process.exitCode = 1;
  }
}

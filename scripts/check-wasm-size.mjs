import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

const workspace = resolve(import.meta.dirname, "..");
const defaultBudgetPath = resolve(workspace, "scripts", "wasm-size-budget.json");

export function checkWasmSize({ wasmPath, budgetPath = defaultBudgetPath } = {}) {
  if (!existsSync(budgetPath)) {
    throw new Error(`WASM size budget is missing: ${budgetPath}`);
  }

  const budget = JSON.parse(readFileSync(budgetPath, "utf8"));
  assert.equal(budget.version, 1, "WASM size budget version must be 1");
  assert.ok(Number.isSafeInteger(budget.maxRawBytes) && budget.maxRawBytes > 0);
  assert.ok(Number.isSafeInteger(budget.maxGzipBytes) && budget.maxGzipBytes > 0);

  const artifact = wasmPath ?? resolve(workspace, budget.artifact);
  if (!existsSync(artifact)) {
    throw new Error(
      `WASM package is missing: ${artifact}. Run node scripts/build-playground.mjs first.`,
    );
  }

  const bytes = readFileSync(artifact);
  const rawBytes = bytes.length;
  const gzipBytes = gzipSync(bytes, { level: 9 }).length;
  const violations = [];
  if (rawBytes > budget.maxRawBytes) {
    violations.push(`raw ${rawBytes} > ${budget.maxRawBytes}`);
  }
  if (gzipBytes > budget.maxGzipBytes) {
    violations.push(`gzip ${gzipBytes} > ${budget.maxGzipBytes}`);
  }
  if (violations.length > 0) {
    throw new Error(`WASM size budget exceeded: ${violations.join(", ")}`);
  }

  return { rawBytes, gzipBytes };
}

function isMainModule() {
  return process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url);
}

if (isMainModule()) {
  try {
    const sizes = checkWasmSize();
    process.stdout.write(
      `WASM size OK: raw=${sizes.rawBytes} bytes, gzip=${sizes.gzipBytes} bytes\n`,
    );
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}

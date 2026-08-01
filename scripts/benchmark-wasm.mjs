import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL, fileURLToPath } from "node:url";
import { performance } from "node:perf_hooks";
import { buildBenchmarkFixtures } from "./benchmark-fixtures.mjs";
import { summarizeDurations } from "./benchmark-metrics.mjs";

const workspace = resolve(import.meta.dirname, "..");

export function measureWasmFixture(lintJson, fixture, {
  warmup,
  iterations,
  now = () => performance.now(),
}) {
  const request = JSON.stringify({
    text: fixture.text,
    sourceKind: fixture.sourceKind,
    config: { profile: "default" },
    includeReviewFixes: false,
  });
  for (let index = 0; index < warmup; index += 1) {
    lintJson(request);
  }

  const samplesMs = [];
  let diagnostics = 0;
  for (let index = 0; index < iterations; index += 1) {
    const start = now();
    const response = JSON.parse(lintJson(request));
    samplesMs.push(now() - start);
    diagnostics = response.diagnostics.length;
  }

  return {
    diagnostics,
    samplesMs,
    summary: summarizeDurations(samplesMs, fixture.byteLength),
  };
}

function positiveInteger(value, name) {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed <= 0) {
    throw new TypeError(`${name} must be a positive integer`);
  }
  return parsed;
}

function parseArguments(arguments_) {
  const options = {
    packageDirectory: resolve(workspace, "apps", "playground", "pkg"),
    warmup: 3,
    iterations: 20,
  };
  for (let index = 0; index < arguments_.length; index += 2) {
    const name = arguments_[index];
    const value = arguments_[index + 1];
    if (value === undefined) {
      throw new TypeError(`${name} requires a value`);
    }
    if (name === "--package-dir") {
      options.packageDirectory = resolve(value);
    } else if (name === "--warmup") {
      options.warmup = positiveInteger(value, "warmup");
    } else if (name === "--iterations") {
      options.iterations = positiveInteger(value, "iterations");
    } else {
      throw new TypeError(`unknown option: ${name}`);
    }
  }
  return options;
}

function gitCommit() {
  try {
    return execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: workspace,
      encoding: "utf8",
    }).trim();
  } catch {
    return process.env.GITHUB_SHA ?? "unknown";
  }
}

async function benchmark(options) {
  const javascriptPath = resolve(options.packageDirectory, "geullint_wasm.js");
  const wasmPath = resolve(options.packageDirectory, "geullint_wasm_bg.wasm");
  const wasmModule = await import(pathToFileURL(javascriptPath).href);
  const wasmBytes = readFileSync(wasmPath);
  const initializationStart = performance.now();
  await wasmModule.default({ module_or_path: wasmBytes });
  const initializeMs = performance.now() - initializationStart;

  const fixtures = buildBenchmarkFixtures().map((fixture) => ({
    id: fixture.id,
    sourceKind: fixture.sourceKind,
    byteLength: fixture.byteLength,
    sha256: fixture.sha256,
    ...measureWasmFixture(wasmModule.lint_json, fixture, options),
  }));

  return {
    schemaVersion: 1,
    runtime: "wasm",
    commit: gitCommit(),
    node: process.version,
    platform: `${process.platform}-${process.arch}`,
    initializeMs: Number(initializeMs.toFixed(3)),
    wasmRawBytes: wasmBytes.byteLength,
    warmup: options.warmup,
    iterations: options.iterations,
    fixtures,
  };
}

const invokedPath = process.argv[1] ? resolve(process.argv[1]) : undefined;
if (invokedPath === fileURLToPath(import.meta.url)) {
  try {
    const report = await benchmark(parseArguments(process.argv.slice(2)));
    process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
  } catch (error) {
    process.stderr.write(
      `GeulLint WASM benchmark: ${error.message}\nBuild first with: node scripts/build-playground.mjs\n`,
    );
    process.exitCode = 2;
  }
}

import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { writeBenchmarkFixtures } from "./benchmark-fixtures.mjs";
import { summarizeDurations } from "./benchmark-metrics.mjs";

const workspace = resolve(import.meta.dirname, "..");

export function cargoFeatureArguments(mode) {
  if (mode === "compact") {
    return ["--no-default-features"];
  }
  if (mode === "source") {
    return [];
  }
  if (mode === "morphology") {
    return ["--features", "morphology"];
  }
  throw new TypeError("native benchmark mode must be compact, source, or morphology");
}

function positiveInteger(value, name) {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed <= 0) {
    throw new TypeError(`${name} must be a positive integer`);
  }
  return parsed;
}

function parseArguments(arguments_) {
  const options = { mode: "source", warmup: 3, iterations: 20 };
  for (let index = 0; index < arguments_.length; index += 2) {
    const name = arguments_[index];
    const value = arguments_[index + 1];
    if (value === undefined) {
      throw new TypeError(`${name} requires a value`);
    }
    if (name === "--mode") {
      cargoFeatureArguments(value);
      options.mode = value;
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

function commandOutput(command, arguments_, options = {}) {
  return execFileSync(command, arguments_, {
    cwd: workspace,
    encoding: "utf8",
    ...options,
  }).trim();
}

function benchmark(options) {
  const targetDirectory = resolve(workspace, process.env.CARGO_TARGET_DIR ?? "target");
  const executable = join(
    targetDirectory,
    "release",
    "examples",
    process.platform === "win32" ? "performance_probe.exe" : "performance_probe",
  );
  const fixturesDirectory = mkdtempSync(join(tmpdir(), "geullint-native-benchmark-"));

  try {
    const manifest = writeBenchmarkFixtures(fixturesDirectory);
    execFileSync("cargo", [
      "build",
      "--locked",
      "--release",
      "-p",
      "geullint-core",
      "--example",
      "performance_probe",
      ...cargoFeatureArguments(options.mode),
    ], { cwd: workspace, stdio: ["ignore", "ignore", "inherit"] });

    const fixtures = manifest.fixtures.map((fixture) => {
      const measured = JSON.parse(commandOutput(executable, [
        "--fixture",
        join(fixturesDirectory, fixture.path),
        "--source-kind",
        fixture.sourceKind,
        "--warmup",
        String(options.warmup),
        "--iterations",
        String(options.iterations),
      ]));
      return {
        id: fixture.id,
        sourceKind: fixture.sourceKind,
        byteLength: fixture.byteLength,
        sha256: fixture.sha256,
        firstCheckMs: measured.firstCheckMs,
        diagnostics: measured.diagnostics,
        samplesMs: measured.samplesMs,
        summary: summarizeDurations(measured.samplesMs, fixture.byteLength),
      };
    });

    return {
      schemaVersion: 1,
      runtime: "native",
      mode: options.mode,
      commit: commandOutput("git", ["rev-parse", "HEAD"]),
      rustc: commandOutput("rustc", ["--version"]),
      platform: `${process.platform}-${process.arch}`,
      probe: basename(executable),
      warmup: options.warmup,
      iterations: options.iterations,
      fixtures,
    };
  } finally {
    rmSync(fixturesDirectory, { recursive: true, force: true });
  }
}

const invokedPath = process.argv[1] ? resolve(process.argv[1]) : undefined;
if (invokedPath === fileURLToPath(import.meta.url)) {
  try {
    const report = benchmark(parseArguments(process.argv.slice(2)));
    process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
  } catch (error) {
    process.stderr.write(`GeulLint native benchmark: ${error.message}\n`);
    process.exitCode = 2;
  }
}

import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { evaluateReviewQuality } from "./evaluate-review-quality.mjs";

function optionValues(arguments_, name) {
  const values = [];
  for (let index = 0; index < arguments_.length; index += 1) {
    if (arguments_[index] === name && arguments_[index + 1]) values.push(arguments_[index + 1]);
  }
  return values;
}

export function evaluateGateReport(report, exitCode = 0, auxiliary = {}) {
  const qualityGate = report?.qualityGate ?? { passed: false, failures: [{ metric: "missingQualityGate" }] };
  const checks = Object.fromEntries(
    Object.entries(auxiliary).map(([name, result]) => [name, result ?? { passed: false }]),
  );
  const auxiliaryPassed = Object.values(checks).every((result) => result?.passed === true);
  return {
    schemaVersion: 1,
    passed: exitCode === 0 && qualityGate.passed === true && auxiliaryPassed,
    qualityGate,
    checks,
    metrics: {
      cases: report?.cases ?? 0,
      precision: report?.precision ?? null,
      recall: report?.recall ?? null,
      specificity: report?.specificity ?? null,
      correctionCases: report?.correctionCases ?? 0,
      top1CorrectionAccuracy: report?.top1CorrectionAccuracy ?? null,
      top5CorrectionAccuracy: report?.top5CorrectionAccuracy ?? null,
      dataset: report?.dataset ?? null,
    },
  };
}

function runCli(arguments_) {
  const corpus = optionValues(arguments_, "--corpus")[0];
  const manifest = optionValues(arguments_, "--manifest")[0];
  const gate = optionValues(arguments_, "--gate")[0];
  const cli = optionValues(arguments_, "--cli")[0] ?? resolve("target", "debug", process.platform === "win32" ? "geullint.exe" : "geullint");
  if ((!corpus && !manifest) || (corpus && manifest) || !gate) throw new Error("usage: node scripts/evaluate-commercial-gate.mjs --corpus PATH|--manifest PATH --gate PATH [--cli PATH]");
  let stdout = "";
  let exitCode = 0;
  try {
    const inputArgs = manifest ? ["--corpus-manifest", manifest] : ["--corpus", corpus];
    stdout = execFileSync(cli, ["--format", "json", ...inputArgs, "--corpus-gate", gate], { encoding: "utf8" });
  } catch (error) {
    stdout = error.stdout?.toString() ?? "";
    exitCode = error.status ?? 2;
  }
  let report;
  try {
    report = JSON.parse(stdout);
  } catch (error) {
    throw new Error(`CLI did not emit a JSON quality report: ${error.message}`);
  }
  const auxiliary = {};
  const leakagePath = optionValues(arguments_, "--leakage")[0];
  if (leakagePath) auxiliary.leakage = JSON.parse(readFileSync(resolve(leakagePath), "utf8"));
  const parityPath = optionValues(arguments_, "--parity")[0];
  if (parityPath) auxiliary.parity = JSON.parse(readFileSync(resolve(parityPath), "utf8"));
  const reviewsPath = optionValues(arguments_, "--review-quality")[0];
  if (reviewsPath) {
    const reviewGatePath = optionValues(arguments_, "--review-gate")[0];
    if (!reviewGatePath) throw new Error("--review-quality requires --review-gate");
    const adjudicationsPath = optionValues(arguments_, "--adjudications")[0];
    const reviews = readFileSync(resolve(reviewsPath), "utf8").split(/\r?\n/u).filter((line) => line.trim()).map(JSON.parse);
    const adjudications = adjudicationsPath
      ? readFileSync(resolve(adjudicationsPath), "utf8").split(/\r?\n/u).filter((line) => line.trim()).map(JSON.parse)
      : [];
    auxiliary.reviewQuality = evaluateReviewQuality(
      reviews,
      adjudications,
      JSON.parse(readFileSync(resolve(reviewGatePath), "utf8")),
    );
  }
  const result = evaluateGateReport(report, exitCode, auxiliary);
  const output = optionValues(arguments_, "--output")[0];
  const serialized = JSON.stringify(result, null, 2);
  if (output) {
    mkdirSync(dirname(resolve(output)), { recursive: true });
    writeFileSync(output, `${serialized}\n`);
  }
  process.stdout.write(`${serialized}\n`);
  if (!result.passed) process.exitCode = 1;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    runCli(process.argv.slice(2));
  } catch (error) {
    console.error(`commercial gate: ${error.message}`);
    process.exitCode = 2;
  }
}

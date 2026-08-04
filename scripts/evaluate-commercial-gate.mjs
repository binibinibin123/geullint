import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

function optionValues(arguments_, name) {
  const values = [];
  for (let index = 0; index < arguments_.length; index += 1) {
    if (arguments_[index] === name && arguments_[index + 1]) values.push(arguments_[index + 1]);
  }
  return values;
}

export function evaluateGateReport(report, exitCode = 0) {
  const qualityGate = report?.qualityGate ?? { passed: false, failures: [{ metric: "missingQualityGate" }] };
  return {
    schemaVersion: 1,
    passed: exitCode === 0 && qualityGate.passed === true,
    qualityGate,
    metrics: {
      cases: report?.cases ?? 0,
      precision: report?.precision ?? null,
      recall: report?.recall ?? null,
      specificity: report?.specificity ?? null,
      dataset: report?.dataset ?? null,
    },
  };
}

function runCli(arguments_) {
  const corpus = optionValues(arguments_, "--corpus")[0];
  const gate = optionValues(arguments_, "--gate")[0];
  const cli = optionValues(arguments_, "--cli")[0] ?? resolve("target", "debug", process.platform === "win32" ? "geullint.exe" : "geullint");
  if (!corpus || !gate) throw new Error("usage: node scripts/evaluate-commercial-gate.mjs --corpus PATH --gate PATH [--cli PATH]");
  let stdout = "";
  let exitCode = 0;
  try {
    stdout = execFileSync(cli, ["--format", "json", "--corpus", corpus, "--corpus-gate", gate], { encoding: "utf8" });
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
  const result = evaluateGateReport(report, exitCode);
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

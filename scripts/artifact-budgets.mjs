import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

function positiveInteger(value, field, name) {
  if (!Number.isSafeInteger(value) || value <= 0) {
    throw new TypeError(`${name}.${field} must be a positive integer`);
  }
  return value;
}

function validateArtifacts(artifacts) {
  if (!Array.isArray(artifacts) || artifacts.length === 0) {
    throw new TypeError("artifact budget must contain at least one artifact");
  }
  for (const artifact of artifacts) {
    if (!artifact || typeof artifact.name !== "string" || artifact.name.trim() === "") {
      throw new TypeError("artifact name must be a non-empty string");
    }
    if (typeof artifact.path !== "string" || artifact.path.trim() === "") {
      throw new TypeError(`${artifact.name}.path must be a non-empty string`);
    }
    positiveInteger(artifact.maxRawBytes, "maxRawBytes", artifact.name);
    positiveInteger(artifact.maxGzipBytes, "maxGzipBytes", artifact.name);
  }
}

export function checkArtifactBudgets({ root = process.cwd(), artifacts }) {
  validateArtifacts(artifacts);
  const measured = artifacts.map((artifact) => {
    const bytes = readFileSync(resolve(root, artifact.path));
    const rawBytes = bytes.byteLength;
    const gzipBytes = gzipSync(bytes, { level: 9 }).byteLength;
    const passed = rawBytes <= artifact.maxRawBytes
      && gzipBytes <= artifact.maxGzipBytes;
    return {
      name: artifact.name,
      path: artifact.path,
      rawBytes,
      gzipBytes,
      maxRawBytes: artifact.maxRawBytes,
      maxGzipBytes: artifact.maxGzipBytes,
      passed,
    };
  });

  return {
    passed: measured.every(({ passed }) => passed),
    artifacts: measured,
  };
}

function runCommand(configPath) {
  const absoluteConfig = resolve(configPath);
  const config = JSON.parse(readFileSync(absoluteConfig, "utf8"));
  if (config.schemaVersion !== 1) {
    throw new TypeError("artifact budget schemaVersion must be 1");
  }
  const root = resolve(dirname(absoluteConfig), config.root ?? ".");
  const report = checkArtifactBudgets({ root, artifacts: config.artifacts });
  process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
  if (!report.passed) {
    process.exitCode = 1;
  }
}

const invokedPath = process.argv[1] ? resolve(process.argv[1]) : undefined;
if (invokedPath === fileURLToPath(import.meta.url)) {
  try {
    runCommand(process.argv[2] ?? "artifact-budgets.json");
  } catch (error) {
    process.stderr.write(`GeulLint artifact budget: ${error.message}\n`);
    process.exitCode = 2;
  }
}

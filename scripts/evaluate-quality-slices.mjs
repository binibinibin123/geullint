import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";

function addSlice(slices, key, value, item) {
  const slice = slices.get(key) ?? { key, value, cases: [] };
  slice.cases.push(item);
  slices.set(key, slice);
}

export function buildQualitySlices(cases) {
  if (!Array.isArray(cases)) throw new TypeError("cases must be an array");
  const slices = new Map();
  for (const item of cases) {
    if (typeof item?.id !== "string" || !item.id.trim()) throw new TypeError("case id must be non-empty");
    for (const dimension of [
      "genre",
      "origin",
      "split",
      "textOrigin",
      "annotationOrigin",
      "annotationStatus",
      "holdoutId",
    ]) {
      const value = typeof item[dimension] === "string" ? item[dimension].trim() : "";
      if (value) addSlice(slices, `${dimension}:${value}`, value, item);
    }
    const families = Array.isArray(item.errorFamilies)
      ? item.errorFamilies.filter((family) => typeof family === "string" && family.trim())
      : [];
    for (const family of families) {
      const value = family.trim();
      addSlice(slices, `errorFamily:${value}`, value, item);
    }
  }
  return [...slices.values()].sort((left, right) => left.key.localeCompare(right.key));
}

export function summarizeQualitySlices(slices) {
  if (!Array.isArray(slices)) throw new TypeError("slices must be an array");
  return slices.map((slice) => {
    const errorCases = slice.cases.filter((item) => item.caseType !== "normal").length;
    return {
      key: slice.key,
      cases: slice.cases.length,
      errorCases,
      normalCases: slice.cases.length - errorCases,
    };
  });
}

async function readCorpus(path) {
  const contents = await readFile(path, "utf8");
  const cases = [];
  for (const [lineIndex, line] of contents.split(/\r?\n/u).entries()) {
    if (!line.trim()) continue;
    try {
      cases.push(JSON.parse(line));
    } catch (error) {
      throw new Error(`${path}:${lineIndex + 1} is not valid JSON: ${error.message}`);
    }
  }
  return cases;
}

async function runCli(arguments_) {
  const index = arguments_.indexOf("--corpus");
  if (index < 0 || !arguments_[index + 1]) {
    throw new Error("usage: node scripts/evaluate-quality-slices.mjs --corpus PATH");
  }
  const cases = await readCorpus(resolve(arguments_[index + 1]));
  const result = {
    cases: cases.length,
    slices: summarizeQualitySlices(buildQualitySlices(cases)),
  };
  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  runCli(process.argv.slice(2)).catch((error) => {
    console.error(`quality slices: ${error.message}`);
    process.exitCode = 2;
  });
}

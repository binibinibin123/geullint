#!/usr/bin/env node
import { createHash } from "node:crypto";
import { mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SOURCE_URL =
  "https://zenodo.org/api/records/16908784/files/KoLLA_multi-refs.m2/content";
const SOURCE_RECORD_URL = "https://zenodo.org/records/16908784";
const SOURCE_MD5 = "9a6f2e3fea1b39bbb7343445db1167f7";
const LICENSE = "GPL-3.0-or-later";

export function allNoopCases(m2) {
  const cases = [];
  const groups = m2.replaceAll("\r\n", "\n").split(/\n\s*\n/u);
  for (const group of groups) {
    const lines = group.split("\n");
    const sourceLine = lines.find((line) => line.startsWith("S "));
    const annotationLines = lines.filter((line) => line.startsWith("A "));
    if (!sourceLine || annotationLines.length === 0) {
      continue;
    }
    const everyAnnotatorSaysNoop = annotationLines.every((line) => {
      const fields = line.slice(2).split("|||");
      return fields[1]?.trim() === "noop";
    });
    if (!everyAnnotatorSaysNoop) {
      continue;
    }
    cases.push({
      id: `kolla-v2-noop-${cases.length + 1}`,
      text: detokenizeM2Sentence(sourceLine.slice(2)),
      sourceKind: "plain_text",
      expectedRuleIds: [],
    });
  }
  return cases;
}

export function correctionReviewQueue(m2) {
  const cases = [];
  const groups = m2.replaceAll("\r\n", "\n").split(/\n\s*\n/u);
  for (const group of groups) {
    const lines = group.split("\n");
    const sourceLine = lines.find((line) => line.startsWith("S "));
    if (!sourceLine) {
      continue;
    }
    const referencesByAnnotator = new Map();
    for (const line of lines.filter((line) => line.startsWith("A "))) {
      const fields = line.slice(2).split("|||");
      const category = fields[1]?.trim();
      if (!category || category === "noop") {
        continue;
      }
      const [start, end] = fields[0]
        .trim()
        .split(/\s+/u)
        .map((value) => Number.parseInt(value, 10));
      const annotator = fields.at(-1)?.trim();
      if (!Number.isInteger(start) || !Number.isInteger(end) || !annotator) {
        continue;
      }
      const reference = referencesByAnnotator.get(annotator) ?? {
        annotator,
        edits: [],
      };
      reference.edits.push({
        startToken: start,
        endToken: end,
        category,
        correction: fields[2]?.trim() === "-NONE-" ? "" : fields[2]?.trim() ?? "",
      });
      referencesByAnnotator.set(annotator, reference);
    }
    if (referencesByAnnotator.size === 0) {
      continue;
    }
    const source = sourceLine.slice(2).trim();
    cases.push({
      id: `kolla-v2-review-${cases.length + 1}`,
      text: detokenizeM2Sentence(source),
      sourceTokens: source.split(/\s+/u),
      references: [...referencesByAnnotator.values()].sort((left, right) =>
        left.annotator.localeCompare(right.annotator),
      ),
    });
  }
  return cases;
}

function detokenizeM2Sentence(sentence) {
  return sentence
    .trim()
    .replace(/\s+([,.;!?])/gu, "$1")
    .replace(/([([{"“‘])\s+/gu, "$1")
    .replace(/\s+([)\]}”’])/gu, "$1");
}

async function main(cliArguments) {
  const accepted = cliArguments.includes("--accept-gpl-3.0-or-later");
  const outputIndex = cliArguments.indexOf("--out-dir");
  const outputDirectory =
    outputIndex === -1 ? undefined : cliArguments[outputIndex + 1];
  if (!accepted || !outputDirectory || outputDirectory.startsWith("--")) {
    throw new Error(
      "Usage: node scripts/acquire-kolla-v2.mjs --accept-gpl-3.0-or-later --out-dir PATH",
    );
  }

  const response = await fetch(SOURCE_URL);
  if (!response.ok) {
    throw new Error(`KoLLA v2 download failed with HTTP ${response.status}`);
  }
  const sourceBytes = Buffer.from(await response.arrayBuffer());
  const sourceMd5 = digest("md5", sourceBytes);
  if (sourceMd5 !== SOURCE_MD5) {
    throw new Error("KoLLA v2 source MD5 does not match the Zenodo record");
  }
  const sourceText = sourceBytes.toString("utf8");
  const cases = allNoopCases(sourceText);
  const reviewQueue = correctionReviewQueue(sourceText);
  if (cases.length === 0) {
    throw new Error("KoLLA v2 did not contain any all-annotator noop sentences");
  }
  if (reviewQueue.length === 0) {
    throw new Error("KoLLA v2 did not contain any reviewable correction cases");
  }

  const normalizedCorpus = `${cases.map(JSON.stringify).join("\n")}\n`;
  const normalizedReviewQueue = `${reviewQueue.map(JSON.stringify).join("\n")}\n`;
  const corpusSha256 = digest("sha256", normalizedCorpus);
  const output = resolve(outputDirectory);
  await mkdir(output, { recursive: true });
  await writeFile(`${output}/KoLLA_multi-refs.m2`, sourceBytes, { flag: "wx" });
  await writeFile(`${output}/kolla-v2-noop.jsonl`, normalizedCorpus, { flag: "wx" });
  await writeFile(`${output}/kolla-v2-review-queue.jsonl`, normalizedReviewQueue, { flag: "wx" });
  await writeFile(
    `${output}/kolla-v2-noop.manifest.json`,
    `${JSON.stringify(
      {
        schemaVersion: 1,
        name: "KoLLA v2 all-annotators-noop controls",
        license: LICENSE,
        sourceUrl: SOURCE_RECORD_URL,
        corpusPath: "kolla-v2-noop.jsonl",
        sha256: corpusSha256,
      },
      null,
      2,
    )}\n`,
    { flag: "wx" },
  );
  console.log(
    JSON.stringify({
      source: SOURCE_RECORD_URL,
      sourceMd5,
      license: LICENSE,
      noopCases: cases.length,
      reviewCases: reviewQueue.length,
      corpusManifest: `${output}/kolla-v2-noop.manifest.json`,
      reviewQueue: `${output}/kolla-v2-review-queue.jsonl`,
    }),
  );
}

function digest(algorithm, value) {
  return createHash(algorithm).update(value).digest("hex");
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(`geullint: ${error.message}`);
    process.exitCode = 2;
  });
}

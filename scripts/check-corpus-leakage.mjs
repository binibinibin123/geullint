import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const DEFAULT_THRESHOLD = 0.85;

export function normalizeCorpusText(value) {
  return String(value)
    .normalize("NFKC")
    .replace(/\s+/gu, " ")
    .trim()
    .toLocaleLowerCase("ko-KR");
}

export function characterNgrams(value, size = 5) {
  const characters = [...normalizeCorpusText(value).normalize("NFD")];
  if (characters.length <= size) return new Set(characters.length ? [characters.join("")] : []);
  const grams = new Set();
  for (let index = 0; index <= characters.length - size; index += 1) {
    grams.add(characters.slice(index, index + size).join(""));
  }
  return grams;
}

function jaccard(left, right) {
  if (left.size === 0 && right.size === 0) return 1;
  let intersection = 0;
  for (const gram of left) if (right.has(gram)) intersection += 1;
  return intersection / (left.size + right.size - intersection);
}

function addIssue(issues, issue) {
  const key = `${issue.kind}:${[issue.leftId, issue.rightId].sort().join("|")}`;
  if (!issues.some((existing) => existing.key === key)) issues.push({ key, ...issue });
}

export function checkCorpusLeakage(
  groups,
  { nearDuplicateThreshold = DEFAULT_THRESHOLD, maxCandidatesPerGram = 512 } = {},
) {
  if (!Array.isArray(groups) || groups.length === 0) {
    throw new TypeError("corpus groups must be a non-empty array");
  }
  if (!(nearDuplicateThreshold > 0 && nearDuplicateThreshold <= 1)) {
    throw new RangeError("nearDuplicateThreshold must be greater than 0 and at most 1");
  }
  if (!(maxCandidatesPerGram === 0 || (Number.isInteger(maxCandidatesPerGram) && maxCandidatesPerGram > 0))) {
    throw new RangeError("maxCandidatesPerGram must be zero or a positive integer");
  }

  const records = [];
  const issues = [];
  const seenIds = new Set();
  for (const group of groups) {
    if (typeof group?.split !== "string" || !group.split.trim()) {
      throw new TypeError("each corpus group requires a non-empty split");
    }
    if (!Array.isArray(group.cases)) throw new TypeError("each corpus group requires cases");
    for (const item of group.cases) {
      if (typeof item?.id !== "string" || !item.id.trim()) throw new TypeError("case id must be non-empty");
      if (typeof item?.text !== "string") throw new TypeError(`case ${item.id} text must be a string`);
      if (seenIds.has(item.id)) throw new Error(`duplicate case id: ${item.id}`);
      seenIds.add(item.id);
      const holdoutId = typeof item.holdoutId === "string" ? item.holdoutId.trim() : null;
      if ((group.split === "H1" || group.split === "H2") && holdoutId !== group.split) {
        addIssue(issues, {
          kind: "holdout_id",
          leftId: item.id,
          rightId: item.id,
          leftSplit: group.split,
          rightSplit: group.split,
          holdoutId,
          expectedHoldoutId: group.split,
        });
      }
      records.push({
        id: item.id,
        text: item.text,
        split: group.split,
        documentId: typeof item.documentId === "string" ? item.documentId.trim() : "",
        authorId: typeof item.authorId === "string" ? item.authorId.trim() : "",
        sourceId: typeof item.sourceId === "string" ? item.sourceId.trim() : "",
        normalized: normalizeCorpusText(item.text),
        grams: characterNgrams(item.text),
      });
    }
  }

  const compareKeys = [
    ["document", "documentId"],
    ["author", "authorId"],
    ["source", "sourceId"],
  ];
  for (const [kind, field] of compareKeys) {
    const byValue = new Map();
    for (const record of records) {
      if (!record[field]) continue;
      const list = byValue.get(record[field]) ?? [];
      list.push(record);
      byValue.set(record[field], list);
    }
    for (const list of byValue.values()) {
      for (let leftIndex = 0; leftIndex < list.length; leftIndex += 1) {
        for (let rightIndex = leftIndex + 1; rightIndex < list.length; rightIndex += 1) {
          const left = list[leftIndex];
          const right = list[rightIndex];
          if (left.split === right.split) continue;
          addIssue(issues, {
            kind,
            leftId: left.id,
            rightId: right.id,
            leftSplit: left.split,
            rightSplit: right.split,
            value: left[field],
          });
        }
      }
    }
  }

  const exact = new Map();
  for (const record of records) {
    const list = exact.get(record.normalized) ?? [];
    for (const previous of list) {
      if (previous.split !== record.split) {
        addIssue(issues, {
          kind: "exact_text",
          leftId: previous.id,
          rightId: record.id,
          leftSplit: previous.split,
          rightSplit: record.split,
        });
      }
    }
    list.push(record);
    exact.set(record.normalized, list);
  }

  const gramIndex = new Map();
  for (const record of records) {
    for (const gram of record.grams) {
      const list = gramIndex.get(gram) ?? [];
      list.push(record);
      gramIndex.set(gram, list);
    }
  }
  const compared = new Set();
  for (const record of records) {
    const candidates = new Set();
    for (const gram of record.grams) {
      const indexed = gramIndex.get(gram) ?? [];
      // Very common grams create a quadratic candidate explosion on large
      // natural-language corpora.  A near duplicate at the configured
      // Jaccard threshold still shares rarer grams; skip only the popular
      // index buckets and keep exact-text checks unconditional.
      const candidateList = maxCandidatesPerGram === 0 || indexed.length <= maxCandidatesPerGram
        ? indexed
        : [];
      for (const candidate of candidateList) {
        if (candidate.id !== record.id && candidate.split !== record.split) candidates.add(candidate);
      }
    }
    for (const candidate of candidates) {
      const pair = [record.id, candidate.id].sort().join("|");
      if (compared.has(pair)) continue;
      compared.add(pair);
      const score = jaccard(record.grams, candidate.grams);
      if (score >= nearDuplicateThreshold) {
        addIssue(issues, {
          kind: "near_duplicate",
          leftId: record.id,
          rightId: candidate.id,
          leftSplit: record.split,
          rightSplit: candidate.split,
          score,
        });
      }
    }
  }

  issues.sort((left, right) => `${left.kind}:${left.leftId}:${left.rightId}`.localeCompare(`${right.kind}:${right.leftId}:${right.rightId}`));
  return {
    passed: issues.length === 0,
    cases: records.length,
    splits: [...new Set(records.map((record) => record.split))].sort(),
    issues: issues.map(({ key, ...issue }) => issue),
  };
}

async function readJsonLines(path, split) {
  const text = await readFile(path, "utf8");
  const cases = [];
  for (const [lineIndex, line] of text.split(/\r?\n/u).entries()) {
    if (!line.trim()) continue;
    let value;
    try {
      value = JSON.parse(line);
    } catch (error) {
      throw new Error(`${path}:${lineIndex + 1} is not valid JSON: ${error.message}`);
    }
    cases.push(value);
  }
  return { split, cases };
}

async function runCli(arguments_) {
  const inputIndex = arguments_.indexOf("--input");
  if (inputIndex < 0 || !arguments_[inputIndex + 1]) {
    throw new Error("usage: node scripts/check-corpus-leakage.mjs --input PATH");
  }
  const inputPath = resolve(arguments_[inputIndex + 1]);
  const input = JSON.parse(await readFile(inputPath, "utf8"));
  if (!Array.isArray(input.corpora)) throw new Error("input.corpora must be an array");
  const groups = await Promise.all(
    input.corpora.map((corpus) => readJsonLines(resolve(dirname(inputPath), corpus.path), corpus.split)),
  );
  const result = checkCorpusLeakage(groups, input.options);
  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
  if (!result.passed) process.exitCode = 1;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  runCli(process.argv.slice(2)).catch((error) => {
    console.error(`corpus leakage: ${error.message}`);
    process.exitCode = 2;
  });
}

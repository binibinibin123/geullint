#!/usr/bin/env node
import { createHash } from "node:crypto";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SOURCE_URLS = {
  tatoeba: "https://tatoeba.org/en/downloads",
  knct: "https://github.com/seonminkoo/K-NCT",
  kolla: "https://zenodo.org/records/16908784",
  kowikitext: "https://github.com/lovit/kowikitext/releases/tag/20200920.v1",
};

const SOURCE_LICENSES = {
  tatoeba: "CC-BY-2.0",
  knct: "NO-REDISTRIBUTION",
  kolla: "GPL-3.0-or-later",
  kowikitext: "CC-BY-SA-3.0",
};

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function sourceMeta(kind, bytes) {
  return {
    sourceId: `public-${kind}`,
    sourceSha256: sha256(bytes),
    sourceUrl: SOURCE_URLS[kind],
    license: SOURCE_LICENSES[kind],
  };
}

function nonblank(value) {
  return typeof value === "string" && value.trim().length > 0;
}

function utf8Range(text, start, end) {
  return {
    start: Buffer.byteLength([...text].slice(0, start).join(""), "utf8"),
    end: Buffer.byteLength([...text].slice(0, end).join(""), "utf8"),
  };
}

function changedSpan(before, after) {
  const left = [...before];
  const right = [...after];
  let start = 0;
  while (start < left.length && start < right.length && left[start] === right[start]) start += 1;
  let leftEnd = left.length;
  let rightEnd = right.length;
  while (leftEnd > start && rightEnd > start && left[leftEnd - 1] === right[rightEnd - 1]) {
    leftEnd -= 1;
    rightEnd -= 1;
  }
  return {
    start,
    end: leftEnd,
    before: left.slice(start, leftEnd).join(""),
    after: right.slice(start, rightEnd).join(""),
  };
}

function reviewProvenance(meta, id, text, corrected, humanEvidence = null) {
  return {
    reviewerType: "human",
    adjudicatorType: null,
    adjudicatorId: null,
    modelSnapshots: [],
    rubricSha256: meta.sourceSha256,
    sessionSha256: sha256(`${meta.sourceId}:source-revision`),
    outputSha256: sha256(JSON.stringify({ id, text, corrected })),
    ...(humanEvidence ? { humanEvidence } : {}),
  };
}

function revisionCase({ id, text, corrected, genre, documentId, authorId, split, meta, family = "source_revision", annotationOrigin = "source_revision", annotationStatus = "reviewed", humanEvidence = null }) {
  const span = changedSpan(text, corrected);
  if (!span.before && !span.after) return null;
  const range = utf8Range(text, span.start, span.end);
  return {
    id,
    text,
    sourceKind: "plain_text",
    caseType: "error",
    expectedFixedText: corrected,
    expectedDiagnostics: [{
      ruleId: "source_revision",
      range,
      suggestions: [span.after],
    }],
    textOrigin: "revision",
    annotationOrigin,
    annotationStatus,
    reviewProvenance: reviewProvenance(meta, id, text, corrected, humanEvidence),
    sourceId: meta.sourceId,
    sourceSha256: meta.sourceSha256,
    sourceUrl: meta.sourceUrl,
    license: meta.license,
    genre,
    documentId,
    authorId,
    split,
    holdoutId: split === "H1" || split === "H2" ? split : null,
    errorFamilies: [family],
  };
}

export function parseTatoebaUsers(tsv) {
  const authors = new Map();
  for (const line of String(tsv).split(/\r?\n/u)) {
    if (!line.trim()) continue;
    const fields = line.split("\t");
    const username = fields[0]?.trim();
    const sentenceId = fields[1]?.trim();
    if (!username || !sentenceId || authors.has(sentenceId)) continue;
    authors.set(sentenceId, username);
  }
  return authors;
}

export function parseTatoebaDetailedAuthors(tsv) {
  const authors = new Map();
  for (const line of String(tsv).split(/\r?\n/u)) {
    if (!line.trim()) continue;
    const fields = line.split("\t");
    const sentenceId = fields[0]?.trim();
    const username = fields[3]?.trim();
    if (!sentenceId || !username || username === "\\N" || authors.has(sentenceId)) continue;
    authors.set(sentenceId, username);
  }
  return authors;
}

export function parseTatoeba(tsv, meta, { limit = Number.POSITIVE_INFINITY } = {}) {
  const cases = [];
  const seen = new Set();
  for (const line of String(tsv).split(/\r?\n/u)) {
    if (!line.trim()) continue;
    const fields = line.split("\t");
    if (fields[1] !== "kor" || !nonblank(fields[2])) continue;
    const text = fields.slice(2).join("\t").normalize("NFKC").trim();
    const normalized = text.replace(/\s+/gu, " ");
    if (!normalized || seen.has(normalized)) continue;
    seen.add(normalized);
    cases.push({
      id: `tatoeba-${fields[0]}`,
      text: normalized,
      sourceKind: "plain_text",
      caseType: "normal",
      expectedDiagnostics: [],
      textOrigin: "human_authored",
      sourceId: meta.sourceId,
      sourceSha256: meta.sourceSha256,
      sourceUrl: meta.sourceUrl,
      license: meta.license,
      genre: "general",
      documentId: `tatoeba:${fields[0]}`,
      authorId: `tatoeba:${meta.authorBySentenceId?.get(fields[0]) ?? "unattributed"}`,
      split: "train",
      holdoutId: null,
      errorFamilies: [],
    });
    if (cases.length >= limit) break;
  }
  return cases;
}

export function parseKowikitext(text, meta, { limit = Number.POSITIVE_INFINITY, offset = 0, split = "H2", holdoutId = split === "H1" || split === "H2" ? split : null } = {}) {
  const cases = [];
  const paragraphs = String(text)
    .split(/(?:\r?\n){2,}/u)
    .map((paragraph) => paragraph.replace(/\s+/gu, " ").trim())
    .filter(nonblank);
  for (let index = offset; index < paragraphs.length && cases.length < limit; index += 1) {
    const paragraph = paragraphs[index];
    cases.push({
      id: `kowikitext-${index + 1}`,
      text: paragraph,
      sourceKind: "plain_text",
      caseType: "normal",
      expectedDiagnostics: [],
      textOrigin: "human_authored",
      sourceId: meta.sourceId,
      sourceSha256: meta.sourceSha256,
      sourceUrl: meta.sourceUrl,
      license: meta.license,
      genre: "kowikitext:encyclopedia",
      documentId: `kowikitext:${index + 1}`,
      authorId: "kowikitext:unattributed",
      split,
      holdoutId,
      errorFamilies: [],
    });
  }
  return cases;
}

export function synthesizeSpacingCorrections(normalCases, { limit = Number.POSITIVE_INFINITY } = {}) {
  const cases = [];
  for (const original of normalCases) {
    if (cases.length >= limit) break;
    const match = /[\p{L}\p{N}] [\p{L}\p{N}]/u.exec(original.text);
    if (!match) continue;
    const spaceOffset = match.index + 1;
    const text = `${original.text.slice(0, spaceOffset)}${original.text.slice(spaceOffset + 1)}`;
    const span = changedSpan(text, original.text);
    const range = utf8Range(text, span.start, span.end);
    const id = `synthetic-spacing-${original.id}`;
    cases.push({
      id,
      text,
      sourceKind: "plain_text",
      caseType: "error",
      expectedFixedText: original.text,
      expectedDiagnostics: [{
        ruleId: "synthetic.spacing",
        range,
        suggestions: [span.after],
      }],
      textOrigin: "synthetic",
      sourceId: original.sourceId,
      sourceSha256: original.sourceSha256,
      sourceUrl: original.sourceUrl,
      license: original.license,
      genre: `synthetic:${original.genre ?? "general"}`,
      documentId: `${original.documentId ?? original.id}:synthetic-spacing`,
      authorId: "geullint:synthetic-spacing-v1",
      split: "train",
      holdoutId: null,
      errorFamilies: ["spacing"],
    });
  }
  return cases;
}

function stripMarkers(value) {
  return String(value).replace(/<e\d+>/gu, "").replace(/<\/e\d+>/gu, "").trim();
}

function familyForKnct(errorType) {
  const values = Object.values(errorType ?? {});
  const value = values[0] ?? "other";
  if (value === "spacing") return "spacing";
  if (value === "punctuation") return "punctuation";
  if (value === "numerical") return "number";
  return "spelling";
}

export function parseKnct(document, meta, { limit = Number.POSITIVE_INFINITY } = {}) {
  const cases = [];
  for (const row of document?.data ?? []) {
    const text = stripMarkers(row.error_sentence);
    const corrected = String(row.correct_sentence ?? "").trim();
    if (!text || !corrected || text === corrected) continue;
    const result = revisionCase({
      id: `knct-${row.index}`,
      text,
      corrected,
      genre: `knct:${String(row.domain ?? "unknown").trim() || "unknown"}`,
      documentId: `knct:${row.index}`,
      authorId: "knct:unattributed",
      split: "H1",
      meta,
      family: familyForKnct(row.error_type),
    });
    if (result) cases.push(result);
    if (cases.length >= limit) break;
  }
  return cases;
}

function detokenize(tokens) {
  return tokens.join(" ")
    .replace(/\s+([,.!?])/gu, "$1")
    .replace(/([([{])\s+/gu, "$1")
    .replace(/\s+([)\]}])/gu, "$1");
}

function applyReference(text, sourceTokens, edits) {
  let corrected = text;
  const ordered = [...edits].sort((left, right) => right.startToken - left.startToken);
  for (const edit of ordered) {
    const before = detokenize(sourceTokens.slice(edit.startToken, edit.endToken));
    const index = corrected.indexOf(before);
    if (index < 0) continue;
    corrected = `${corrected.slice(0, index)}${edit.correction}${corrected.slice(index + before.length)}`;
  }
  return corrected;
}

export function parseKollaQueue(rows, meta, { limit = Number.POSITIVE_INFINITY, expandReferences = false } = {}) {
  const cases = [];
  for (const row of rows) {
    const references = expandReferences ? (row.references ?? []) : (row.references?.slice(0, 1) ?? []);
    for (const reference of references) {
      if (!reference || !Array.isArray(reference.edits) || reference.edits.length === 0) continue;
      const corrected = applyReference(row.text, row.sourceTokens ?? [], reference.edits);
      if (!corrected || corrected === row.text) continue;
      const independent = expandReferences && (row.references?.length ?? 0) > 1;
      const id = expandReferences ? `kolla-${row.id}-ref-${reference.annotator}` : `kolla-${row.id}`;
      const result = revisionCase({
        id,
        text: row.text,
        corrected,
        genre: "kolla:learner",
        documentId: `kolla:${row.id}`,
        authorId: "kolla:unattributed",
        split: "H2",
        meta,
        family: reference.edits.some((edit) => String(edit.category).toUpperCase().includes("SPELL")) ? "spelling" : "grammar",
        annotationOrigin: independent ? "human_independent" : "source_revision",
        annotationStatus: "reviewed",
        humanEvidence: independent ? {
          evidenceId: id,
          sourceUrl: meta.sourceUrl,
          annotatorId: String(reference.annotator ?? "unknown"),
          referenceCount: row.references.length,
        } : null,
      });
      if (result) cases.push(result);
      if (cases.length >= limit) break;
    }
    if (cases.length >= limit) break;
  }
  return cases;
}

export function buildEvaluationBundle({ tatoeba = [], knct = [], kolla = [], kowikitext = [], synthetic = [], safety = [] }) {
  const candidates = [...tatoeba, ...knct, ...kolla, ...kowikitext, ...synthetic, ...safety]
    .filter((entry) => entry && typeof entry.id === "string" && typeof entry.text === "string")
    .sort((left, right) => left.id.localeCompare(right.id));
  const splitPriority = { H1: 0, H2: 1, train: 2 };
  const casePriority = (entry) => [entry.caseType === "error" ? 0 : 1, splitPriority[entry.split ?? "train"] ?? 3, entry.id];
  const byNormalizedText = new Map();
  for (const entry of candidates) {
    const key = entry.text.normalize("NFKC").replace(/\s+/gu, " ").trim().toLocaleLowerCase("ko-KR");
    const group = byNormalizedText.get(key) ?? [];
    group.push(entry);
    byNormalizedText.set(key, group);
  }
  const cases = [];
  let crossSplitTextDeduplicated = 0;
  for (const group of byNormalizedText.values()) {
    const splits = new Set(group.map((entry) => entry.split ?? "train"));
    if (splits.size <= 1) {
      cases.push(...group);
      continue;
    }
    const winner = [...group].sort((left, right) => casePriority(left).join("\u0000").localeCompare(casePriority(right).join("\u0000")))[0];
    const winningSplit = winner.split ?? "train";
    for (const entry of group) {
      if ((entry.split ?? "train") === winningSplit) cases.push(entry);
      else crossSplitTextDeduplicated += 1;
    }
  }
  cases.sort((left, right) => left.id.localeCompare(right.id));
  const ids = new Set();
  for (const entry of cases) {
    if (ids.has(entry.id)) throw new Error(`duplicate evaluation case id: ${entry.id}`);
    ids.add(entry.id);
  }
  const report = {
    schemaVersion: 1,
    counts: {
      total: cases.length,
      normal: cases.filter((entry) => entry.caseType === "normal").length,
      errors: cases.filter((entry) => entry.caseType === "error").length,
      sourceRevision: cases.filter((entry) => entry.annotationOrigin === "source_revision").length,
      independentHuman: cases.filter((entry) => entry.annotationOrigin === "human_independent").length,
      synthetic: cases.filter((entry) => entry.textOrigin === "synthetic").length,
      crossSplitTextDeduplicated,
    },
    sources: [...new Set(cases.map((entry) => entry.sourceId).filter(Boolean))].sort(),
    splits: Object.fromEntries([...new Set(cases.map((entry) => entry.split).filter(Boolean))].sort().map((split) => [split, cases.filter((entry) => entry.split === split).length])),
  };
  return { cases, report };
}

async function readJsonLines(path) {
  const text = await readFile(resolve(path), "utf8");
  return text.split(/\r?\n/u).filter((line) => line.trim()).map((line) => JSON.parse(line));
}

function parseJsonLinesText(text) {
  return String(text).split(/\r?\n/u).filter((line) => line.trim()).map((line) => JSON.parse(line));
}

function arg(args, name, required = true) {
  const index = args.indexOf(name);
  const value = index < 0 ? undefined : args[index + 1];
  if (required && (!value || value.startsWith("--"))) throw new Error(`missing ${name}`);
  return value && !value.startsWith("--") ? value : undefined;
}

async function main(args) {
  const tatoebaPath = arg(args, "--tatoeba");
  const tatoebaSourcePath = arg(args, "--tatoeba-source");
  const tatoebaUsersPath = arg(args, "--tatoeba-users", false);
  const tatoebaDetailedPath = arg(args, "--tatoeba-detailed", false);
  const knctPath = arg(args, "--knct");
  const kollaPath = arg(args, "--kolla");
  const kollaSourcePath = arg(args, "--kolla-source");
  const kowikitextPath = arg(args, "--kowikitext", false);
  const kowikitextSourcePath = arg(args, "--kowikitext-source", false);
  const syntheticCorrections = Number(arg(args, "--synthetic-corrections", false) ?? 0);
  const safetyPath = arg(args, "--safety", false);
  const outputDirectory = arg(args, "--out-dir");
  const [tatoebaBytes, tatoebaSourceBytes, tatoebaUsersBytes, tatoebaDetailedBytes, knctBytes, kollaBytes, kollaSourceBytes, kowikitextBytes, kowikitextSourceBytes, safety] = await Promise.all([
    readFile(resolve(tatoebaPath)),
    readFile(resolve(tatoebaSourcePath)),
    tatoebaUsersPath ? readFile(resolve(tatoebaUsersPath)) : Promise.resolve(null),
    tatoebaDetailedPath ? readFile(resolve(tatoebaDetailedPath)) : Promise.resolve(null),
    readFile(resolve(knctPath)),
    readFile(resolve(kollaPath)),
    readFile(resolve(kollaSourcePath)),
    kowikitextPath ? readFile(resolve(kowikitextPath)) : Promise.resolve(null),
    kowikitextSourcePath ? readFile(resolve(kowikitextSourcePath)) : Promise.resolve(null),
    safetyPath ? readJsonLines(safetyPath) : Promise.resolve([]),
  ]);
  const tatoebaMeta = sourceMeta("tatoeba", tatoebaSourceBytes);
  const authorMetadataBytes = tatoebaDetailedBytes ?? tatoebaUsersBytes;
  const tatoebaAuthorBySentenceId = tatoebaDetailedBytes
    ? parseTatoebaDetailedAuthors(tatoebaDetailedBytes.toString("utf8"))
    : (tatoebaUsersBytes ? parseTatoebaUsers(tatoebaUsersBytes.toString("utf8")) : new Map());
  if (authorMetadataBytes) {
    tatoebaMeta.authorMetadataSha256 = sha256(authorMetadataBytes);
    tatoebaMeta.authorMetadataSourceUrl = SOURCE_URLS.tatoeba;
  }
  const knctMeta = sourceMeta("knct", knctBytes);
  const kollaMeta = sourceMeta("kolla", kollaSourceBytes);
  const kowikitextMeta = kowikitextBytes && kowikitextSourceBytes ? sourceMeta("kowikitext", kowikitextSourceBytes) : null;
  const tatoeba = parseTatoeba(tatoebaBytes.toString("utf8"), { ...tatoebaMeta, authorBySentenceId: tatoebaAuthorBySentenceId });
  const result = buildEvaluationBundle({
    tatoeba,
    knct: parseKnct(JSON.parse(knctBytes.toString("utf8")), knctMeta),
    kolla: parseKollaQueue(parseJsonLinesText(kollaBytes.toString("utf8")), kollaMeta, { expandReferences: true }),
    synthetic: syntheticCorrections > 0 ? synthesizeSpacingCorrections(tatoeba, { limit: syntheticCorrections }) : [],
    ...(kowikitextMeta ? {
      kowikitext: parseKowikitext(kowikitextBytes.toString("utf8"), kowikitextMeta, { limit: 600, split: "H2" }),
    } : {}),
    safety,
  });
  await mkdir(resolve(outputDirectory), { recursive: true });
  const corpus = `${result.cases.map(JSON.stringify).join("\n")}\n`;
  const corpusPath = resolve(outputDirectory, "public-evaluation-v1.jsonl");
  await writeFile(corpusPath, corpus, "utf8");
  const splitFiles = {};
  const splitNames = ["train", "H1", "H2"];
  if (result.cases.some((entry) => entry.split === "release_holdout")) splitNames.push("release_holdout");
  for (const split of splitNames) {
    const splitCases = result.cases.filter((entry) => (entry.split ?? "train") === split);
    const fileName = `public-evaluation-v1.${split}.jsonl`;
    await writeFile(resolve(outputDirectory, fileName), `${splitCases.map(JSON.stringify).join("\n")}\n`, "utf8");
    splitFiles[split] = fileName;
  }
  const leakageInput = {
    schemaVersion: 1,
    options: { nearDuplicateThreshold: 0.85 },
    corpora: Object.entries(splitFiles).map(([split, fileName]) => ({ path: fileName, split })),
  };
  await writeFile(resolve(outputDirectory, "public-evaluation-v1.leakage.json"), `${JSON.stringify(leakageInput, null, 2)}\n`, "utf8");
  const manifest = {
    schemaVersion: 1,
    name: "GeulLint public local evaluation bundle v1",
    license: "MIXED_LOCAL_ONLY",
    sourceUrl: "https://github.com/binibinibin123/geullint/blob/codex/ai-adjudicated-eval/docs/public-evaluation.md",
    corpusPath: "public-evaluation-v1.jsonl",
    sha256: sha256(corpus),
    splitFiles,
    sources: [tatoebaMeta, knctMeta, kollaMeta, ...(kowikitextMeta ? [kowikitextMeta] : [])].map((entry) => {
      const sanitized = { ...entry };
      delete sanitized.authorBySentenceId;
      return sanitized;
    }),
  };
  await writeFile(resolve(outputDirectory, "public-evaluation-v1.manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
  await writeFile(resolve(outputDirectory, "public-evaluation-v1.report.json"), `${JSON.stringify(result.report, null, 2)}\n`, "utf8");
  process.stdout.write(`${JSON.stringify({ ...result.report, corpusPath }, null, 2)}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(`public evaluation bundle: ${error.message}`);
    process.exitCode = 2;
  });
}

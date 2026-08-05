import test from "node:test";
import assert from "node:assert/strict";

import {
  buildEvaluationBundle,
  parseKnct,
  parseKollaQueue,
  parseKowikitext,
  parseTatoebaDetailedAuthors,
  parseTatoebaUsers,
  parseTatoeba,
  synthesizeSpacingCorrections,
} from "./build-public-evaluation-bundle.mjs";

const source = {
  sourceId: "test-source",
  sourceSha256: "a".repeat(64),
  sourceUrl: "https://example.com/source",
  license: "CC-BY-4.0",
};

test("parses Tatoeba Korean rows into normal cases with provenance fields", () => {
  const cases = parseTatoeba("1\tkor\t첫 문장입니다.\n2\teng\tEnglish.\n1\tkor\t첫 문장입니다.\n", source);
  assert.equal(cases.length, 1);
  assert.equal(cases[0].caseType, "normal");
  assert.equal(cases[0].text, "첫 문장입니다.");
  assert.equal(cases[0].sourceId, "test-source");
  assert.equal(cases[0].split, "train");
});

test("parses K-NCT corrections into exact source-revision cases", () => {
  const cases = parseKnct({ data: [{
    index: 7,
    error_sentence: "나는 밥을 <e1>먹엇다</e1>.",
    correct_sentence: "나는 밥을 먹었다.",
    domain: "daily",
  }] }, source);
  assert.equal(cases.length, 1);
  assert.equal(cases[0].caseType, "error");
  assert.equal(cases[0].expectedFixedText, "나는 밥을 먹었다.");
  assert.equal(cases[0].annotationOrigin, "source_revision");
  assert.equal(cases[0].expectedDiagnostics.length, 1);
  assert.deepEqual(cases[0].expectedDiagnostics[0].suggestions, ["었"]);
  assert.equal(cases[0].split, "H1");
});

test("parses a KoLLA review queue without inventing an adjudicator", () => {
  const cases = parseKollaQueue([{
    id: "kolla-1",
    text: "나는 밥을 먹엇다.",
    sourceTokens: ["나는", "밥을", "먹엇다", "."],
    references: [{ annotator: "0", edits: [{ startToken: 2, endToken: 3, correction: "먹었다" }] }],
  }], source);
  assert.equal(cases.length, 1);
  assert.equal(cases[0].expectedFixedText, "나는 밥을 먹었다.");
  assert.equal(cases[0].split, "H2");
  assert.equal(cases[0].reviewProvenance.reviewerType, "human");
});

test("builds a deterministic bundle and retains mixed source counts", () => {
  const result = buildEvaluationBundle({
    tatoeba: parseTatoeba("1\tkor\t정상 문장입니다.\n", source),
    knct: parseKnct({ data: [{ index: 1, error_sentence: "틀린 문장.", correct_sentence: "바른 문장.", domain: "news" }] }, source),
    kolla: parseKollaQueue([], source),
    safety: [],
  });
  assert.equal(result.cases.length, 2);
  assert.deepEqual(result.report.counts, {
    total: 2,
    normal: 1,
    errors: 1,
    sourceRevision: 1,
    independentHuman: 0,
    crossSplitTextDeduplicated: 0,
    synthetic: 0,
  });
  assert.equal(result.cases[0].id, "knct-1");
});

test("marks generated spacing perturbations as synthetic instead of human review", () => {
  const normal = parseTatoeba("1\tkor\t오늘 은 맑다.\n", source);
  const cases = synthesizeSpacingCorrections(normal, { limit: 1 });
  assert.equal(cases.length, 1);
  assert.equal(cases[0].textOrigin, "synthetic");
  assert.equal(cases[0].expectedFixedText, "오늘 은 맑다.");
  assert.notEqual(cases[0].text, cases[0].expectedFixedText);
});

test("keeps a source revision and removes a cross-split normal duplicate", () => {
  const result = buildEvaluationBundle({
    tatoeba: [{ id: "tatoeba-1", text: "같은 문장", caseType: "normal", split: "train" }],
    knct: [{
      id: "knct-1",
      text: "같은 문장",
      caseType: "error",
      annotationOrigin: "source_revision",
      split: "H1",
    }],
  });
  assert.deepEqual(result.cases.map((entry) => entry.id), ["knct-1"]);
  assert.equal(result.report.counts.crossSplitTextDeduplicated, 1);
});

test("maps Tatoeba sentence authors from the users export", () => {
  const authors = parseTatoebaUsers("alice\t1\t1\t2020-01-01\n\tbogus\n");
  const cases = parseTatoeba("1\tkor\t안녕하세요.\n", { ...source, authorBySentenceId: authors });
  assert.equal(cases[0].authorId, "tatoeba:alice");
});

test("maps Tatoeba detailed-export authors without treating the sentence text as metadata", () => {
  const authors = parseTatoebaDetailedAuthors("1\tkor\t문장입니다.\twriter\t\\N\t2024-01-01\n");
  const cases = parseTatoeba("1\tkor\t문장입니다.\n", { ...source, authorBySentenceId: authors });
  assert.equal(cases[0].authorId, "tatoeba:writer");
});

test("parses a CC-BY-SA KWikiText line as an H2 normal case", () => {
  const cases = parseKowikitext("첫 번째 문장입니다.\n\n두 번째 문장입니다.\n", source, { limit: 1 });
  assert.equal(cases.length, 1);
  assert.equal(cases[0].caseType, "normal");
  assert.equal(cases[0].split, "H2");
  assert.equal(cases[0].textOrigin, "human_authored");
});

test("can reserve a separate release holdout from the same licensed source", () => {
  const cases = parseKowikitext("첫 문장입니다.\n\n둘째 문장입니다.\n", source, { limit: 1, split: "release_holdout", holdoutId: null });
  assert.equal(cases[0].split, "release_holdout");
  assert.equal(cases[0].holdoutId, null);
});

test("expands KoLLA multi-reference rows into independent human annotations", () => {
  const rows = [{
    id: "kolla-2",
    text: "문장 오류",
    sourceTokens: ["문장", "오류"],
    references: [
      { annotator: "0", edits: [{ startToken: 1, endToken: 2, correction: "수정" }] },
      { annotator: "1", edits: [{ startToken: 1, endToken: 2, correction: "교정" }] },
    ],
  }];
  const cases = parseKollaQueue(rows, source, { expandReferences: true });
  assert.equal(cases.length, 2);
  assert.deepEqual(cases.map((entry) => entry.id), ["kolla-kolla-2-ref-0", "kolla-kolla-2-ref-1"]);
  assert.equal(cases[0].annotationOrigin, "human_independent");
  assert.equal(cases[0].annotationStatus, "reviewed");
  assert.equal(cases[0].reviewProvenance.humanEvidence.evidenceId, "kolla-kolla-2-ref-0");
});

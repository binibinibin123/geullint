import test from "node:test";
import assert from "node:assert/strict";

import {
  buildEvaluationBundle,
  parseKnct,
  parseKollaQueue,
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

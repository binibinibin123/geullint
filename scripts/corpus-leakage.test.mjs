import assert from "node:assert/strict";
import test from "node:test";
import { characterNgrams, checkCorpusLeakage } from "./check-corpus-leakage.mjs";

test("rejects exact text and source document leakage across splits", () => {
  const result = checkCorpusLeakage([
    {
      split: "train",
      cases: [
        {
          id: "train-1",
          text: "같은 문장이 두 분할에 들어갔습니다.",
          documentId: "doc-shared",
          authorId: "author-shared",
        },
      ],
    },
    {
      split: "release_holdout",
      cases: [
        {
          id: "holdout-1",
          text: "같은 문장이 두 분할에 들어갔습니다.",
          documentId: "doc-shared",
          authorId: "author-shared",
        },
      ],
    },
  ]);

  assert.equal(result.passed, false);
  assert.ok(result.issues.some((issue) => issue.kind === "exact_text"));
  assert.ok(result.issues.some((issue) => issue.kind === "document"));
  assert.ok(result.issues.some((issue) => issue.kind === "author"));
});

test("rejects near duplicate Korean sentences with a high five-gram overlap", () => {
  const result = checkCorpusLeakage([
    {
      split: "dev",
      cases: [
        {
          id: "dev-1",
          text: "오늘 회의에서 새 검사기의 품질과 성능을 자세히 검토했습니다.",
        },
      ],
    },
    {
      split: "release_holdout",
      cases: [
        {
          id: "holdout-1",
          text: "오늘 회의에서 새 검사기의 품질과 성능을 자세히 검토했습니다!",
        },
      ],
    },
  ]);

  assert.equal(result.passed, false);
  assert.ok(result.issues.some((issue) => issue.kind === "near_duplicate"));
});

test("accepts distinct source documents and texts", () => {
  const result = checkCorpusLeakage([
    {
      split: "train",
      cases: [
        { id: "train-1", text: "첫 번째 문서는 독립된 내용을 담습니다.", documentId: "doc-a" },
      ],
    },
    {
      split: "release_holdout",
      cases: [
        { id: "holdout-1", text: "두 번째 문서는 완전히 다른 예시입니다.", documentId: "doc-b" },
      ],
    },
  ]);

  assert.equal(result.passed, true);
  assert.deepEqual(result.issues, []);
});

test("uses decomposed Hangul jamo for five-gram leakage checks", () => {
  const grams = characterNgrams("한글 맞춤법");
  assert.ok([...grams].some((gram) => gram.includes("ᄒ")), "expected decomposed choseong in the gram index");
});

test("does not drop a near duplicate after a popular gram exceeds the candidate cap", () => {
  const trainCases = Array.from({ length: 513 }, (_, index) => ({
    id: `train-${index}`,
    text: `공통문자열-${index}-독립 문서의 서로 다른 꼬리표입니다.`,
  }));
  const result = checkCorpusLeakage([
    { split: "train", cases: trainCases },
    {
      split: "H1",
      cases: [{
        id: "holdout-target",
        text: "공통문자열-512-독립 문서의 서로 다른 꼬리표입니다.",
      }],
    },
  ], { maxCandidatesPerGram: 0 });
  assert.equal(result.passed, false);
  assert.ok(result.issues.some((issue) => issue.kind === "near_duplicate" && issue.rightId === "holdout-target"));
});

test("rejects source lineage leakage and a mismatched holdout identifier", () => {
  const result = checkCorpusLeakage([
    {
      split: "dev",
      cases: [{ id: "dev-source", text: "개발 문장입니다.", sourceId: "source-shared" }],
    },
    {
      split: "H1",
      cases: [{ id: "holdout-source", text: "잠금 문장입니다.", sourceId: "source-shared", holdoutId: "H2" }],
    },
  ]);
  assert.equal(result.passed, false);
  assert.ok(result.issues.some((issue) => issue.kind === "source"));
  assert.ok(result.issues.some((issue) => issue.kind === "holdout_id"));
});

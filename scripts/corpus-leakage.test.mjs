import assert from "node:assert/strict";
import test from "node:test";
import { checkCorpusLeakage } from "./check-corpus-leakage.mjs";

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

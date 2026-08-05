import assert from "node:assert/strict";
import test from "node:test";

import { splitCorpusByDocument } from "./split-corpus-by-document.mjs";

const cases = [
  { id: "a-1", text: "문서 A의 첫 문장입니다.", textOrigin: "human_authored", documentId: "doc-a", authorId: "author-a", sourceId: "source-a" },
  { id: "a-2", text: "문서 A의 둘째 문장입니다.", textOrigin: "human_authored", documentId: "doc-b", authorId: "author-a", sourceId: "source-a" },
  { id: "b-1", text: "문서 B의 첫 문장입니다.", textOrigin: "human_authored", documentId: "doc-c", authorId: "author-b", sourceId: "source-b" },
  { id: "c-1", text: "문서 C의 첫 문장입니다.", textOrigin: "human_authored", documentId: "doc-d", authorId: "author-c", sourceId: "source-c" },
];

test("splits deterministically and keeps one author in one split", () => {
  const first = splitCorpusByDocument(cases, { seed: "test-seed" });
  const second = splitCorpusByDocument(cases, { seed: "test-seed" });
  assert.deepEqual(first, second);
  const authorSplits = new Map();
  for (const entry of first) {
    const previous = authorSplits.get(entry.authorId);
    if (previous) assert.equal(entry.split, previous);
    authorSplits.set(entry.authorId, entry.split);
    if (entry.split === "H1" || entry.split === "H2") assert.equal(entry.holdoutId, entry.split);
  }
});

test("rejects non-project rows without document and author provenance", () => {
  assert.throws(
    () => splitCorpusByDocument([{ ...cases[0], documentId: "" }]),
    /requires documentId/u,
  );
  assert.throws(
    () => splitCorpusByDocument([{ ...cases[0], authorId: "" }]),
    /requires authorId/u,
  );
});

test("rejects duplicate IDs and existing cross-split leakage", () => {
  assert.throws(
    () => splitCorpusByDocument([cases[0], cases[0]]),
    /duplicate case id/u,
  );
  assert.throws(
    () => splitCorpusByDocument([
      { ...cases[0], split: "train" },
      { ...cases[1], split: "H1" },
    ]),
    /author appears in multiple splits/u,
  );
});

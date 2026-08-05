import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import test from "node:test";

import {
  importEvaluationSource,
  verifyEvaluationSource,
} from "./import-evaluation-sources.mjs";

const bytes = Buffer.from("원문 출처\n", "utf8");
const source = {
  id: "public-source",
  url: "https://example.test/source.jsonl",
  recordUrl: "https://example.test/record",
  license: "CC-BY-4.0",
  sha256: createHash("sha256").update(bytes).digest("hex"),
  access: "public",
  redistributable: true,
};

test("verifies a locked source before importing evaluation candidates", () => {
  assert.deepEqual(verifyEvaluationSource(source, bytes), {
    sourceId: "public-source",
    sha256: source.sha256,
    license: "CC-BY-4.0",
    sourceUrl: "https://example.test/record",
  });
  const imported = importEvaluationSource(
    [{
      id: "case-1",
      text: "검토할 원문입니다.",
      genre: "news",
      documentId: "doc-1",
      authorId: "author-1",
    }],
    {
      source,
      bytes,
      split: "H1",
      textOrigin: "human_authored",
      forbiddenDocumentIds: [],
    },
  );
  assert.deepEqual(imported[0], {
    id: "case-1",
    text: "검토할 원문입니다.",
    genre: "news",
    documentId: "doc-1",
    authorId: "author-1",
    sourceId: "public-source",
    sourceSha256: source.sha256,
    sourceUrl: "https://example.test/record",
    license: "CC-BY-4.0",
    textOrigin: "human_authored",
    annotationOrigin: "source_revision",
    annotationStatus: "unreviewed",
    split: "H1",
    holdoutId: "H1",
  });
});

test("rejects manual authorization, hash mismatch, and unsafe holdout rows", () => {
  assert.throws(
    () => verifyEvaluationSource({ ...source, access: "manual_authorization" }, bytes),
    /manual authorization/u,
  );
  assert.throws(
    () => verifyEvaluationSource({ ...source, sha256: "0".repeat(64) }, bytes),
    /SHA-256 does not match/u,
  );
  assert.throws(
    () => importEvaluationSource(
      [{ id: "synthetic", text: "x", genre: "news", documentId: "doc", authorId: "author" }],
      { source, bytes, split: "H2", textOrigin: "synthetic" },
    ),
    /synthetic.*holdout/u,
  );
  assert.throws(
    () => importEvaluationSource(
      [{ id: "train-doc", text: "x", genre: "news", documentId: "kolla-train-1", authorId: "author" }],
      { source, bytes, split: "H1", forbiddenDocumentIds: ["kolla-train-1"] },
    ),
    /training document/u,
  );
});

import assert from "node:assert/strict";
import test from "node:test";

import {
  planDownloads,
  validateSourceManifest,
} from "./acquire-training-data.mjs";
import {
  extractRevisionEdits,
  changedSpan,
} from "./extract-human-edits.mjs";

const manifest = {
  schemaVersion: 1,
  sources: [
    {
      id: "public-demo",
      url: "https://example.test/demo.jsonl",
      recordUrl: "https://example.test/record",
      license: "CC-BY-4.0",
      licenseUrl: "https://creativecommons.org/licenses/by/4.0/",
      sha256: "a".repeat(64),
      filename: "public-demo.jsonl",
      redistributable: true,
    },
    {
      id: "review-only",
      url: "https://example.test/review.m2",
      recordUrl: "https://example.test/review",
      license: "GPL-3.0-or-later",
      licenseUrl: "https://www.gnu.org/licenses/gpl-3.0.html",
      sha256: "b".repeat(64),
      filename: "review.m2",
      redistributable: false,
    },
  ],
};

test("validates a source manifest and blocks unsafe paths", () => {
  assert.deepEqual(validateSourceManifest(manifest), manifest);
  assert.throws(
    () => validateSourceManifest({ ...manifest, sources: [{ ...manifest.sources[0], filename: "../escape" }] }),
    /filename must be a relative file name/,
  );
  assert.throws(
    () => validateSourceManifest({ ...manifest, sources: [{ ...manifest.sources[0], license: "Unknown" }] }),
    /license is not on the allowlist/,
  );
});

test("requires explicit acceptance for non-redistributable sources", () => {
  assert.deepEqual(planDownloads(manifest), [manifest.sources[0]]);
  assert.deepEqual(planDownloads(manifest, { acceptNonRedistributable: true }), manifest.sources);
  assert.throws(() => planDownloads(manifest, { includeNonRedistributable: true }), /explicit acceptance/);
});

test("extracts a single human edit with provenance and a review status", () => {
  const result = extractRevisionEdits([
    {
      id: "rev-1",
      documentId: "doc-1",
      authorId: "author-1",
      genre: "essay",
      sourceUrl: "https://example.test/doc-1",
      license: "CC-BY-4.0",
      before: "회의가 몇일 뒤에 열립니다.",
      after: "회의가 며칠 뒤에 열립니다.",
    },
  ]);
  assert.deepEqual(result, [
    {
      id: "revision-rev-1",
      text: "회의가 몇일 뒤에 열립니다.",
      expectedFixedText: "회의가 며칠 뒤에 열립니다.",
      origin: "revision",
      split: "train",
      genre: "essay",
      documentId: "doc-1",
      authorId: "author-1",
      provenanceId: "rev-1",
      sourceUrl: "https://example.test/doc-1",
      license: "CC-BY-4.0",
      errorFamilies: ["revision.unclassified"],
      reviewStatus: "needs_adjudication",
      changed: { before: "몇일", after: "며칠" },
    },
  ]);
});

test("rejects no-op, multi-span, and oversized revisions", () => {
  assert.deepEqual(
    extractRevisionEdits([
      { id: "noop", documentId: "d", authorId: "a", genre: "essay", sourceUrl: "https://example.test/u", license: "CC-BY-4.0", before: "같다", after: "같다" },
      { id: "multi", documentId: "d", authorId: "a", genre: "essay", sourceUrl: "https://example.test/u", license: "CC-BY-4.0", before: "가 나 다", after: "나 라 다" },
      { id: "large", documentId: "d", authorId: "a", genre: "essay", sourceUrl: "https://example.test/u", license: "CC-BY-4.0", before: "가".repeat(100), after: "나".repeat(100) },
    ]),
    [],
  );
  assert.deepEqual(changedSpan("abc", "axc"), { before: "b", after: "x", start: 1, end: 2 });
});

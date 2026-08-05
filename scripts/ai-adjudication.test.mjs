import assert from "node:assert/strict";
import test from "node:test";

import {
  mergeBlindReviews,
  validateCaseV2,
} from "./ai-adjudication.mjs";

const hash = (letter) => letter.repeat(64);

function baseCase(overrides = {}) {
  return {
    id: "case-1",
    text: "회의가 끝났다.",
    textOrigin: "human_authored",
    annotationOrigin: "ai_blind_panel",
    annotationStatus: "reviewed",
    genre: "workplace",
    split: "dev",
    documentId: "doc-1",
    authorId: "author-1",
    holdoutId: null,
    expectedDiagnostics: [],
    reviewProvenance: {
      reviewerType: "ai",
      adjudicatorType: "ai",
      modelSnapshots: ["gpt-5.6-sol@2026-08-05", "gpt-5.6-terra@2026-08-05"],
      rubricSha256: hash("a"),
      sessionSha256: hash("b"),
      outputSha256: hash("c"),
    },
    ...overrides,
  };
}

test("rejects AI annotation mislabeled as independent human review", () => {
  assert.throws(
    () => validateCaseV2(baseCase({ annotationOrigin: "human_independent" })),
    /AI review cannot be labeled independent_human/u,
  );
  assert.throws(
    () => validateCaseV2(baseCase({ annotationOrigin: "ai_blind_panel", reviewProvenance: { ...baseCase().reviewProvenance, reviewerType: "human" } })),
    /AI annotation requires reviewerType ai/u,
  );
});

test("accepts a reviewed normal case with no expected diagnostics", () => {
  assert.deepEqual(validateCaseV2(baseCase()), baseCase());
});

test("accepts an ambiguous adjudication with no forced correction", () => {
  const ambiguous = baseCase({ annotationStatus: "ambiguous" });
  assert.deepEqual(validateCaseV2(ambiguous), ambiguous);
});

test("merges unanimous blind reviews without exposing engine output", () => {
  const base = {
    id: "case-2",
    text: "몇일 뒤에 만나요.",
    textOrigin: "human_authored",
    genre: "dialogue",
    split: "dev",
    documentId: "doc-2",
    authorId: "author-2",
    holdoutId: null,
  };
  const reviews = ["a", "b", "c"].map((reviewerId) => ({
    reviewerId,
    reviewerType: "ai",
    modelSnapshot: `model-${reviewerId}`,
    rubricSha256: hash("d"),
    sessionSha256: hash(reviewerId),
    outputSha256: hash("e"),
    status: "error",
    diagnostics: [{
      range: { start: 0, end: 6 },
      suggestions: ["며칠"],
      errorFamily: "spelling",
    }],
  }));
  const result = mergeBlindReviews(base, reviews);
  assert.equal(result.annotationOrigin, "ai_blind_panel");
  assert.equal(result.annotationStatus, "reviewed");
  assert.equal(result.expectedDiagnostics[0].suggestions[0], "며칠");
  assert.equal(result.reviewProvenance.reviewerType, "ai");
});

test("requires adjudication for conflicting blind reviews and preserves ambiguity", () => {
  const base = {
    id: "case-3",
    text: "이 표현은 문맥에 따라 다르다.",
    textOrigin: "human_authored",
    genre: "education",
    split: "H1",
    documentId: "doc-3",
    authorId: "author-3",
    holdoutId: "H1",
  };
  const reviews = ["a", "b"].map((reviewerId, index) => ({
    reviewerId,
    reviewerType: "ai",
    modelSnapshot: `model-${reviewerId}`,
    rubricSha256: hash("f"),
    sessionSha256: hash(reviewerId),
    outputSha256: hash("g"),
    status: index === 0 ? "normal" : "error",
    diagnostics: index === 0 ? [] : [{ range: { start: 0, end: 3 }, suggestions: ["다른"], errorFamily: "style" }],
  }));
  assert.throws(() => mergeBlindReviews(base, reviews), /adjudication required/u);
  const result = mergeBlindReviews(base, reviews, {
    reviewerId: "d",
    reviewerType: "ai",
    modelSnapshot: "model-adjudicator",
    rubricSha256: hash("f"),
    sessionSha256: hash("h"),
    outputSha256: hash("i"),
    status: "ambiguous",
    diagnostics: [],
  });
  assert.equal(result.annotationStatus, "ambiguous");
  assert.deepEqual(result.expectedDiagnostics, []);
  assert.equal(result.reviewProvenance.adjudicatorType, "ai");
});

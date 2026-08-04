import assert from "node:assert/strict";
import test from "node:test";
import { buildQualitySlices, summarizeQualitySlices } from "./evaluate-quality-slices.mjs";

const cases = [
  {
    id: "news-1",
    text: "몇일 뒤에 만나요.",
    genre: "news",
    origin: "independent_human",
    split: "release_holdout",
    caseType: "error",
    errorFamilies: ["spelling"],
  },
  {
    id: "news-2",
    text: "오늘 문서를 읽는다.",
    genre: "news",
    origin: "independent_human",
    split: "release_holdout",
    caseType: "normal",
    errorFamilies: [],
  },
  {
    id: "tech-1",
    text: "끝낼수 없습니다.",
    genre: "technical",
    origin: "revision",
    split: "dev",
    caseType: "error",
    errorFamilies: ["spacing", "grammar"],
  },
];

test("builds deterministic genre, origin, split, and error-family slices", () => {
  const slices = buildQualitySlices(cases);
  assert.deepEqual(
    slices.map((slice) => slice.key),
    [
      "errorFamily:grammar",
      "errorFamily:spacing",
      "errorFamily:spelling",
      "genre:news",
      "genre:technical",
      "origin:independent_human",
      "origin:revision",
      "split:dev",
      "split:release_holdout",
    ],
  );
  assert.equal(slices.find((slice) => slice.key === "genre:news").cases.length, 2);
  assert.equal(slices.find((slice) => slice.key === "errorFamily:spacing").cases.length, 1);
});

test("summarizes normal and error cases without losing slice identity", () => {
  const summary = summarizeQualitySlices(buildQualitySlices(cases));
  assert.deepEqual(summary, [
    { key: "errorFamily:grammar", cases: 1, errorCases: 1, normalCases: 0 },
    { key: "errorFamily:spacing", cases: 1, errorCases: 1, normalCases: 0 },
    { key: "errorFamily:spelling", cases: 1, errorCases: 1, normalCases: 0 },
    { key: "genre:news", cases: 2, errorCases: 1, normalCases: 1 },
    { key: "genre:technical", cases: 1, errorCases: 1, normalCases: 0 },
    { key: "origin:independent_human", cases: 2, errorCases: 1, normalCases: 1 },
    { key: "origin:revision", cases: 1, errorCases: 1, normalCases: 0 },
    { key: "split:dev", cases: 1, errorCases: 1, normalCases: 0 },
    { key: "split:release_holdout", cases: 2, errorCases: 1, normalCases: 1 },
  ]);
});

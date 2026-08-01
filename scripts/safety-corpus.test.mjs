import assert from "node:assert/strict";
import test from "node:test";

import { validateSafetyCorpus } from "./validate-safety-corpus.mjs";

const policy = {
  schemaVersion: 1,
  exactCaseCount: 4,
  minErrorCases: 2,
  minNormalCases: 2,
  requiredGenres: ["news", "chat"],
  minCasesPerGenre: 2,
  requiredSourceKinds: ["plain_text", "markdown"],
  minProfiles: 2,
  minimumProfileCounts: { default: 2, strict: 2 },
  maxNormalizedDuplicateCount: 0,
  maxChar3GramJaccardSimilarity: 0.82,
  minTextLength: 10,
  requiredRuleIds: ["spelling.safe", "spelling.review"],
  highRiskRuleMinimums: { "spelling.safe": 1 },
};

function validCases() {
  return [
    {
      id: "news-error",
      text: "회의 일정은 몇일 뒤로 확정됐습니다.",
      genre: "news",
      sourceKind: "plain_text",
      profile: "default",
      caseType: "error",
      provenanceId: "authored-news-001",
      expectedDiagnostics: [
        {
          ruleId: "spelling.safe",
          original: "몇일",
          suggestions: ["며칠"],
        },
      ],
      expectedFixedText: "회의 일정은 며칠 뒤로 확정됐습니다.",
    },
    {
      id: "chat-review",
      text: "오늘 저녁에는 찌게를 같이 먹을래?",
      genre: "chat",
      sourceKind: "markdown",
      profile: "strict",
      caseType: "error",
      provenanceId: "authored-chat-001",
      expectedDiagnostics: [
        {
          ruleId: "spelling.review",
          original: "찌게",
          suggestions: ["찌개"],
        },
      ],
      expectedFixedText: "오늘 저녁에는 찌게를 같이 먹을래?",
    },
    {
      id: "news-normal",
      text: "지역 도서관은 다음 달에 다시 문을 엽니다.",
      genre: "news",
      sourceKind: "plain_text",
      profile: "default",
      caseType: "normal",
      provenanceId: "authored-news-002",
      expectedDiagnostics: [],
      expectedFixedText: "지역 도서관은 다음 달에 다시 문을 엽니다.",
    },
    {
      id: "chat-normal",
      text: "업데이트 끝나면 변경 사항을 댓글로 알려 줘.",
      genre: "chat",
      sourceKind: "markdown",
      profile: "strict",
      caseType: "normal",
      provenanceId: "authored-chat-002",
      expectedDiagnostics: [],
      expectedFixedText: "업데이트 끝나면 변경 사항을 댓글로 알려 줘.",
    },
  ];
}

function validate(cases, policyOverride = {}) {
  return validateSafetyCorpus({
    jsonl: cases.map((entry) => JSON.stringify(entry)).join("\n"),
    policy: { ...policy, ...policyOverride },
    knownRuleIds: new Set(["spelling.safe", "spelling.review"]),
  });
}

function messages(result) {
  return result.errors.join("\n");
}

test("accepts a compact, diverse safety regression fixture", () => {
  const result = validate(validCases());
  assert.equal(result.valid, true, messages(result));
  assert.deepEqual(result.summary, {
    cases: 4,
    errorCases: 2,
    normalCases: 2,
    genres: 2,
    sourceKinds: 2,
    profiles: 2,
    normalizedDuplicateCount: 0,
  });
});

test("rejects duplicate IDs and whitespace-normalized duplicate text", () => {
  const cases = validCases();
  cases[1].id = cases[0].id;
  cases[3].text = `  ${cases[2].text.replaceAll(" ", "   ")}  `;
  cases[3].expectedFixedText = cases[3].text;
  const result = validate(cases);
  assert.equal(result.valid, false);
  assert.match(messages(result), /duplicate id/u);
  assert.match(messages(result), /duplicate normalized text/u);
});

test("allows normalized duplicates up to the configured limit", () => {
  const cases = validCases();
  cases[3].text = `  ${cases[2].text.replaceAll(" ", "   ")}  `;
  cases[3].expectedFixedText = cases[3].text;
  const result = validate(cases, { maxNormalizedDuplicateCount: 1 });
  assert.equal(result.valid, true, messages(result));
  assert.equal(result.summary.normalizedDuplicateCount, 1);
});

test("rejects near-template sentences above the 3-gram similarity limit", () => {
  const cases = validCases();
  cases[3].text = "지역 도서관은 다음 달에 다시 문을 열었습니다.";
  cases[3].expectedFixedText = cases[3].text;
  const result = validate(cases, { maxChar3GramJaccardSimilarity: 0.5 });
  assert.equal(result.valid, false);
  assert.match(messages(result), /3-gram Jaccard similarity/u);
  assert.match(
    messages(result),
    /case `news-normal` \(line 3\) and case `chat-normal` \(line 4\)/u,
  );
});

test("rejects missing genre, source kind, and profile coverage", async (t) => {
  for (const [field, pattern] of [
    ["genre", /genre/u],
    ["sourceKind", /sourceKind/u],
    ["profile", /profile/u],
  ]) {
    await t.test(field, () => {
      const cases = validCases();
      delete cases[0][field];
      const result = validate(cases);
      assert.equal(result.valid, false);
      assert.match(messages(result), pattern);
    });
  }
});

test("enforces minimum counts for each named profile", () => {
  const result = validate(validCases(), {
    minimumProfileCounts: { default: 3, strict: 1 },
  });
  assert.equal(result.valid, false);
  assert.match(messages(result), /profile `default` has 2 cases; requires at least 3/u);
});

test("rejects unknown rules and unmet high-risk positive counts", () => {
  const unknownCases = validCases();
  unknownCases[0].expectedDiagnostics[0].ruleId = "spelling.unknown";
  const unknownResult = validate(unknownCases);
  assert.equal(unknownResult.valid, false);
  assert.match(messages(unknownResult), /unknown ruleId/u);

  const countResult = validate(validCases(), {
    highRiskRuleMinimums: { "spelling.safe": 2 },
  });
  assert.equal(countResult.valid, false);
  assert.match(messages(countResult), /high-risk positive/u);
});

test("rejects an empty or ambiguous original and missing suggestions", () => {
  for (const original of ["", "몇일"]) {
    const cases = validCases();
    cases[0].text = original === "" ? cases[0].text : "몇일 뒤 몇일 안에 회의를 엽니다.";
    cases[0].expectedDiagnostics[0].original = original;
    const result = validate(cases);
    assert.equal(result.valid, false);
    assert.match(messages(result), /original/u);
  }

  const cases = validCases();
  cases[0].expectedDiagnostics[0].suggestions = [];
  const result = validate(cases);
  assert.equal(result.valid, false);
  assert.match(messages(result), /suggestions/u);
});

test("rejects normal/error annotation mismatches", () => {
  const normalCases = validCases();
  normalCases[0].caseType = "normal";
  let result = validate(normalCases);
  assert.equal(result.valid, false);
  assert.match(messages(result), /normal case/u);

  const errorCases = validCases();
  errorCases[2].caseType = "error";
  result = validate(errorCases);
  assert.equal(result.valid, false);
  assert.match(messages(result), /error case/u);
});

test("accepts unchanged expectedFixedText for a review-only annotation shape", () => {
  const cases = validCases();
  assert.equal(cases[1].expectedFixedText, cases[1].text);
  const result = validate(cases);
  assert.equal(result.valid, true, messages(result));
});

test("accepts an explicit UTF-8 range with a deletion suggestion", () => {
  const cases = validCases();
  cases[0].text = "회의를 마쳤다  .다음 일정을 정했습니다.";
  const stringStart = cases[0].text.indexOf("  .");
  const byteStart = Buffer.byteLength(cases[0].text.slice(0, stringStart));
  cases[0].expectedDiagnostics = [
    {
      ruleId: "spelling.safe",
      original: "  ",
      range: { start: byteStart, end: byteStart + 2 },
      suggestions: [""],
    },
  ];
  cases[0].expectedFixedText = "회의를 마쳤다.다음 일정을 정했습니다.";

  const result = validate(cases);

  assert.equal(result.valid, true, messages(result));
});

test("rejects an inconsistent changed expectedFixedText", () => {
  const cases = validCases();
  cases[0].expectedFixedText = "회의 일정은 몇 일 뒤로 확정됐습니다.";
  const result = validate(cases);
  assert.equal(result.valid, false);
  assert.match(messages(result), /expectedFixedText/u);
});

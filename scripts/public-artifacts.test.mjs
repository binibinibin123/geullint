import assert from "node:assert/strict";
import test from "node:test";
import {
  buildRuleMarkdown,
  buildSmokeCorpus,
  validateCatalog,
} from "./generate-public-artifacts.mjs";

const catalog = {
  version: 1,
  ruleCount: 2,
  rules: [
    {
      id: "spacing.fixed.sample",
      title: "띄어쓰기",
      description: "표현을 띄어 씁니다.",
      category: "spacing",
      confidence: "high",
      defaultEnabled: true,
      fixSafety: "safe",
      profiles: ["default", "strict", "editorial"],
      incorrectExamples: ["할수"],
      correctExamples: ["할 수"],
      documentationUrl: "https://example.test/rules#spacing.fixed.sample",
    },
    {
      id: "style.sample",
      title: "문체",
      description: "겹말을 다듬습니다.",
      category: "style",
      confidence: "medium",
      defaultEnabled: false,
      fixSafety: "review",
      profiles: ["editorial"],
      incorrectExamples: ["무료 사은품"],
      correctExamples: ["사은품"],
      documentationUrl: "https://example.test/rules#style.sample",
    },
  ],
};

test("validates and renders one deterministic smoke case per rule", () => {
  assert.doesNotThrow(() => validateCatalog(catalog));
  const rendered = buildSmokeCorpus(catalog);
  const rows = rendered.trimEnd().split("\n").map(JSON.parse);

  assert.equal(rows.length, 2);
  assert.deepEqual(rows[0].expectedRuleIds, ["spacing.fixed.sample"]);
  assert.equal(rows[1].profile, "editorial");
  assert.equal(rendered, buildSmokeCorpus(catalog));
});

test("renders matching Markdown anchors and examples", () => {
  const rendered = buildRuleMarkdown(catalog);

  assert.match(rendered, /^# GeulLint 규칙 2개/m);
  assert.equal(rendered.match(/<a id="/g).length, 2);
  assert.match(rendered, /`할수` → `할 수`/);
});

test("keeps the paired -던지 matcher executable as two diagnostics", () => {
  const pairedCatalog = {
    version: 1,
    ruleCount: 1,
    rules: [{
      ...catalog.rules[0],
      id: "grammar.ending.deun-choice",
      incorrectExamples: ["커피던지 차던지"],
      correctExamples: ["커피든지 차든지"],
    }],
  };
  const [row] = buildSmokeCorpus(pairedCatalog).trimEnd().split("\n").map(JSON.parse);

  assert.deepEqual(row.expectedRuleIds, [
    "grammar.ending.deun-choice",
    "grammar.ending.deun-choice",
  ]);
});

test("rejects count drift, unsorted IDs, and incomplete examples", () => {
  assert.throws(
    () => validateCatalog({ ...catalog, ruleCount: 3 }),
    /ruleCount/,
  );
  assert.throws(
    () => validateCatalog({ ...catalog, rules: [...catalog.rules].reverse() }),
    /sorted/,
  );
  assert.throws(
    () => validateCatalog({
      ...catalog,
      rules: [{ ...catalog.rules[0], incorrectExamples: [] }, catalog.rules[1]],
    }),
    /examples/,
  );
});

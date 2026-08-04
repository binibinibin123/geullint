import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

import { createRuleQuickPickItems, type RuleCatalog } from "../rule-catalog";

const catalog: RuleCatalog = {
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
      title: "겹말",
      description: "겹치는 뜻을 다듬습니다.",
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

test("maps offline rule metadata into searchable Quick Pick items", () => {
  const items = createRuleQuickPickItems(catalog);

  assert.equal(items.length, 2);
  assert.equal(items[0].label, "띄어쓰기");
  assert.match(items[0].description, /spacing\.fixed\.sample/);
  assert.match(items[0].detail, /할수 → 할 수/);
  assert.equal(items[1].rule.defaultEnabled, false);
});

test("declares the rule browser command in the extension manifest", () => {
  const manifest = JSON.parse(fs.readFileSync("package.json", "utf8")) as {
    contributes: { commands: Array<{ command: string; title: string }> };
  };

  /* assert.deepEqual(manifest.contributes.commands, [{
    command: "geullint.openRuleCatalog",
    title: "GeulLint: 규칙 목록 열기",
  }]); */
  assert.deepEqual(
    manifest.contributes.commands.map(({ command }) => command).sort(),
    ["geullint.fixAllSafe", "geullint.openRuleCatalog"],
  );
});

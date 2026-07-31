export interface RuleMetadata {
  id: string;
  title: string;
  description: string;
  category: string;
  confidence: "high" | "medium" | "low";
  defaultEnabled: boolean;
  fixSafety: "safe" | "review" | "none";
  profiles: Array<"default" | "strict" | "editorial">;
  incorrectExamples: string[];
  correctExamples: string[];
  documentationUrl: string;
}

export interface RuleCatalog {
  version: number;
  ruleCount: number;
  rules: RuleMetadata[];
}

export interface RuleQuickPickItem {
  label: string;
  description: string;
  detail: string;
  rule: RuleMetadata;
}

export function createRuleQuickPickItems(catalog: RuleCatalog): RuleQuickPickItem[] {
  if (catalog.version !== 1 || catalog.ruleCount !== catalog.rules.length) {
    throw new Error("GeulLint 규칙 카탈로그 버전 또는 개수가 올바르지 않습니다.");
  }
  return catalog.rules.map((rule) => {
    const incorrect = rule.incorrectExamples[0] ?? "";
    const correct = rule.correctExamples[0] ?? "";
    const enabled = rule.defaultEnabled ? "기본 활성" : "선택 활성";
    return {
      label: rule.title,
      description: `${rule.id} · ${rule.category} · ${enabled}`,
      detail: `${incorrect} → ${correct} — ${rule.description}`,
      rule,
    };
  });
}

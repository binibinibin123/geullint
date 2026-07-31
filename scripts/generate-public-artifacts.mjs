import { execFileSync } from "node:child_process";
import { writeFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";

function requireText(value, label) {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${label} must be non-empty text`);
  }
}

export function validateCatalog(catalog) {
  if (catalog?.version !== 1 || !Array.isArray(catalog.rules)) {
    throw new Error("catalog must use version 1 and contain rules");
  }
  if (catalog.ruleCount !== catalog.rules.length) {
    throw new Error("catalog ruleCount must match rules.length");
  }

  let previousId = "";
  const ids = new Set();
  for (const rule of catalog.rules) {
    requireText(rule.id, "rule id");
    requireText(rule.title, `${rule.id} title`);
    requireText(rule.description, `${rule.id} description`);
    if (rule.id <= previousId) {
      throw new Error("catalog rule IDs must be unique and sorted");
    }
    previousId = rule.id;
    ids.add(rule.id);
    if (
      !Array.isArray(rule.incorrectExamples)
      || rule.incorrectExamples.length === 0
      || !Array.isArray(rule.correctExamples)
      || rule.correctExamples.length === 0
    ) {
      throw new Error(`${rule.id} must contain incorrect and correct examples`);
    }
    requireText(rule.incorrectExamples[0], `${rule.id} incorrect example`);
    requireText(rule.correctExamples[0], `${rule.id} correct example`);
  }
  if (ids.size !== catalog.ruleCount) {
    throw new Error("catalog rule IDs must be unique");
  }
}

export function buildSmokeCorpus(catalog) {
  validateCatalog(catalog);
  const lines = catalog.rules.map((rule) => {
    const diagnosticCount = rule.id === "grammar.ending.deun-choice" ? 2 : 1;
    return JSON.stringify({
      id: `bundled-${rule.id}`,
      text: rule.incorrectExamples[0],
      sourceKind: "plain_text",
      profile: "editorial",
      expectedRuleIds: Array(diagnosticCount).fill(rule.id),
    });
  });
  return `${lines.join("\n")}\n`;
}

function escapeInlineCode(value) {
  return value.replaceAll("`", "\\`");
}

export function buildRuleMarkdown(catalog) {
  validateCatalog(catalog);
  const lines = [
    `# GeulLint 규칙 ${catalog.ruleCount}개`,
    "",
    "> 이 파일은 공개 규칙 카탈로그에서 재현 가능하게 생성됩니다.",
    "",
  ];
  for (const rule of catalog.rules) {
    lines.push(`<a id="${rule.id}"></a>`);
    lines.push(`## \`${rule.id}\` — ${rule.title}`);
    lines.push("");
    lines.push(rule.description);
    lines.push("");
    lines.push(`- 분류: \`${rule.category}\``);
    lines.push(`- 신뢰도: \`${rule.confidence}\``);
    lines.push(`- 수정 안전도: \`${rule.fixSafety}\``);
    lines.push(`- 기본 활성화: \`${rule.defaultEnabled}\``);
    lines.push(`- 프로필: ${rule.profiles.map((profile) => `\`${profile}\``).join(", ")}`);
    lines.push(
      `- 예: \`${escapeInlineCode(rule.incorrectExamples[0])}\` → `
      + `\`${escapeInlineCode(rule.correctExamples[0])}\``,
    );
    lines.push("");
  }
  return lines.join("\n");
}

async function main(arguments_) {
  if (arguments_.length !== 3) {
    throw new Error(
      "Usage: node scripts/generate-public-artifacts.mjs GEULLINT_BINARY CORPUS_PATH DOCS_PATH",
    );
  }
  const [binaryPath, corpusPath, docsPath] = arguments_;
  const stdout = execFileSync(binaryPath, ["rules", "--format", "json"], {
    encoding: "utf8",
    windowsHide: true,
  });
  const catalog = JSON.parse(stdout);
  validateCatalog(catalog);
  await Promise.all([
    writeFile(corpusPath, buildSmokeCorpus(catalog), "utf8"),
    writeFile(docsPath, buildRuleMarkdown(catalog), "utf8"),
  ]);
  process.stdout.write(
    `generated ${catalog.ruleCount} corpus cases and rule sections\n`,
  );
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  main(process.argv.slice(2)).catch((error) => {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 2;
  });
}

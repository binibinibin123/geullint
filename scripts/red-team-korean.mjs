import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";

export const RED_TEAM_CASES = [
  { id: "proper-github", text: "GitHub 저장소의 README를 확인했다.", expected: [] },
  { id: "proper-product", text: "GeulLint 설정을 저장했다.", expected: [] },
  { id: "quoted-word", text: "문서에는 `몇일`이라는 예시가 있다.", sourceKind: "markdown", expected: [] },
  { id: "url", text: "https://example.com/몇일 경로를 확인했다.", expected: [] },
  { id: "code-string", text: "const label = \"몇일\";", sourceKind: "javascript", expected: [] },
  { id: "mixed-language", text: "API response를 확인했다.", expected: [] },
  { id: "normal-myeochil", text: "며칠 뒤에 만나요.", expected: [] },
  { id: "typo-myeochil", text: "몇일 뒤에 만나요.", expected: ["spelling.lexical.myeochil"] },
  { id: "typo-geumse", text: "금새 알려 드릴게요.", expected: ["spelling.lexical.geumse"] },
  { id: "typo-wenman", text: "왠만하면 참석하겠습니다.", expected: ["spelling.confusable.wen-waen"] },
  { id: "spacing-al-su", text: "결과를 알수없다.", expected: ["spacing.dependent-noun.su"] },
  { id: "punctuation-comma", text: "시간,, 상태를 확인한다.", expected: ["punctuation.duplicate.comma"] },
];

export function evaluateRedTeamResults(results) {
  const failures = results.filter((result) => !result.passed);
  return {
    schemaVersion: 1,
    cases: results.length,
    passed: failures.length === 0,
    failures,
  };
}

function run(arguments_) {
  const cliIndex = arguments_.indexOf("--cli");
  const cli = cliIndex >= 0 ? arguments_[cliIndex + 1] : resolve("target", "debug", process.platform === "win32" ? "geullint.exe" : "geullint");
  if (!cli) throw new Error("usage: node scripts/red-team-korean.mjs --cli PATH");
  const directory = mkdtempSync(join(tmpdir(), "geullint-red-team-"));
  try {
    const results = RED_TEAM_CASES.map((fixture) => {
      const extension = fixture.sourceKind === "markdown" ? ".md" : fixture.sourceKind ? ".js" : ".txt";
      const path = join(directory, `${fixture.id}${extension}`);
      writeFileSync(path, fixture.text);
      const args = ["--format", "json", path];
      let output;
      try {
        output = execFileSync(cli, args, { encoding: "utf8" });
      } catch (error) {
        output = error.stdout?.toString() ?? "{}";
      }
      const report = JSON.parse(output);
      const actual = (report.diagnostics ?? []).map((diagnostic) => diagnostic.ruleId);
      const expected = fixture.expected;
      return {
        id: fixture.id,
        expected,
        actual,
        passed: expected.every((rule) => actual.includes(rule)) && actual.every((rule) => expected.includes(rule)),
      };
    });
    const report = evaluateRedTeamResults(results);
    process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
    if (!report.passed) process.exitCode = 1;
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    run(process.argv.slice(2));
  } catch (error) {
    console.error(`Korean red team: ${error.message}`);
    process.exitCode = 2;
  }
}

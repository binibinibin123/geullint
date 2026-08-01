import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const version = "0.3.0-alpha.1";
const tag = `v${version}`;

const packagePaths = [
  "packages/npm/geullint/package.json",
  "packages/npm/geullint-win32-x64/package.json",
  "packages/npm/geullint-win32-arm64/package.json",
  "packages/npm/geullint-darwin-x64/package.json",
  "packages/npm/geullint-darwin-arm64/package.json",
  "packages/npm/geullint-linux-x64/package.json",
  "packages/npm/geullint-linux-arm64/package.json",
  "extensions/vscode-geullint/package.json",
  "extensions/vscode-geullint/package-lock.json",
];

test("keeps release metadata on one version", () => {
  assert.match(
    readFileSync("Cargo.toml", "utf8"),
    new RegExp(`^version = "${version.replaceAll(".", "\\.")}"$`, "mu"),
  );

  for (const path of packagePaths) {
    const metadata = JSON.parse(readFileSync(path, "utf8"));
    assert.equal(metadata.version, version, path);
    if (metadata.packages?.[""]) {
      assert.equal(metadata.packages[""].version, version, `${path} root package`);
    }
  }

  const launcher = JSON.parse(readFileSync("packages/npm/geullint/package.json", "utf8"));
  for (const dependencyVersion of Object.values(launcher.optionalDependencies)) {
    assert.equal(dependencyVersion, version);
  }

  assert.match(readFileSync("CITATION.cff", "utf8"), new RegExp(`^version: ${version}$`, "mu"));
  assert.match(readFileSync("CHANGELOG.md", "utf8"), new RegExp(`^## \\[${version.replaceAll(".", "\\.")}\\]`, "mu"));
});

test("pins public install examples to the audited tag", () => {
  const readmes = ["README.md", "README.en.md", "README.ja.md", "README.zh-CN.md"];
  for (const path of readmes) {
    const source = readFileSync(path, "utf8");
    assert.match(source, new RegExp(tag.replaceAll(".", "\\."), "u"), path);
    assert.match(source, new RegExp(`raw\\.githubusercontent\\.com/binibinibin123/geullint/${tag.replaceAll(".", "\\.")}/install\\.ps1`, "u"), `${path} PowerShell installer`);
    assert.match(source, new RegExp(`raw\\.githubusercontent\\.com/binibinibin123/geullint/${tag.replaceAll(".", "\\.")}/install\\.sh`, "u"), `${path} POSIX installer`);
  }
});

test("ships a versioned quality report with measured limits", () => {
  const report = readFileSync(`docs/quality-report-v${version}.md`, "utf8");

  assert.match(report, /공개 규칙: 113개/u);
  assert.match(report, /오류 문장 \| 72/u);
  assert.match(report, /정상 문장 \| 72/u);
  assert.match(report, /정상 문장 \| 249/u);
  assert.match(report, /범용 OOV 철자 사전은 기본 엔진에 없습니다/u);
  assert.match(report, /Harper나 상용 맞춤법 검사기와 동급이라고 부르지 않습니다/u);
});

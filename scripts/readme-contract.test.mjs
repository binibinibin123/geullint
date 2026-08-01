import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const readme = readFileSync("README.md", "utf8");
const distributionGuide = readFileSync("docs/distribution.md", "utf8");
const dictionaryOverlayGuide = readFileSync("docs/dictionary-overlay.md", "utf8");
const corpusEvaluationGuide = readFileSync("docs/corpus-evaluation.md", "utf8");
const corpusSourcesGuide = readFileSync("docs/corpus-sources.md", "utf8");
const rulePackGuide = readFileSync("docs/rule-packs.md", "utf8");
const extensionGuide = readFileSync("extensions/vscode-geullint/README.md", "utf8");
const localizedReadmes = [
  ["README.md", readme],
  ["README.en.md", readFileSync("README.en.md", "utf8")],
  ["README.ja.md", readFileSync("README.ja.md", "utf8")],
  ["README.zh-CN.md", readFileSync("README.zh-CN.md", "utf8")],
];

test("leads with a spelling-checker product promise and direct action", () => {
  const productPromises = [
    /글을 밖으로 보내지 않는 오픈소스 한국어 맞춤법 검사기/u,
    /Open-source Korean spelling and grammar checker that keeps your writing local/iu,
    /文章を外部へ送らない、オープンソースの韓国語文章校正・文法チェック/u,
    /不上传文本的开源韩语拼写与语法检查器/u,
  ];
  const directActions = [
    /지금 문장 검사하기/u,
    /Check a sentence now/iu,
    /今すぐ文章をチェック/u,
    /立即检查句子/u,
  ];
  const useCaseHeadings = [
    /## 어디서나 같은 맞춤법 검사/u,
    /## One checker, wherever you write/iu,
    /## 書く場所を選ばない校正/u,
    /## 在任何写作场景中使用/u,
  ];
  const dictionaryTerms = [
    /사용자 사전/u,
    /user dictionar/iu,
    /ユーザー辞書/u,
    /用户词典/u,
  ];

  localizedReadmes.forEach(([path, source], index) => {
    const demoOffset = source.indexOf("assets/demo/geullint-demo.gif");
    assert.ok(demoOffset > 0, `${path} places a demo after the introduction`);
    const introduction = source.slice(0, demoOffset);
    assert.match(introduction, productPromises[index], `${path} names the product first`);
    assert.match(introduction, directActions[index], `${path} offers an immediate check`);
    assert.match(source, useCaseHeadings[index], `${path} explains writing use cases`);
    assert.match(source, /VS Code/u, `${path} covers real-time editor checks`);
    assert.match(source, /CLI/u, `${path} covers batch checks`);
    assert.match(source, /CI/u, `${path} covers document quality gates`);
    assert.match(source, dictionaryTerms[index], `${path} covers custom dictionaries`);
  });
});

test("keeps the four README translations aligned", () => {
  for (const [path, source] of localizedReadmes) {
    assert.match(source, /README\.md/iu, `${path} links Korean`);
    assert.match(source, /README\.en\.md/iu, `${path} links English`);
    assert.match(source, /README\.ja\.md/iu, `${path} links Japanese`);
    assert.match(source, /README\.zh-CN\.md/iu, `${path} links Chinese`);
    assert.match(source, /assets\/demo\/geullint-demo\.gif/u, `${path} shows the demo`);
    assert.match(source, /assets\/screenshots\/vscode\.png/u, `${path} shows the editor`);
    assert.match(source, /binibinibin123\.github\.io\/geullint/u, `${path} links the live playground`);
    assert.match(source, /github\.com\/binibinibin123\/geullint\/releases/u, `${path} links releases`);
    assert.match(source, /0\.3\.0-alpha\.1/u, `${path} identifies the alpha release`);
    assert.match(source, /Windows/iu, `${path} documents Windows`);
    assert.match(source, /macOS/iu, `${path} documents macOS`);
    assert.match(source, /Linux/iu, `${path} documents Linux`);
    assert.match(source, /offline|오프라인|オフライン|离线/iu, `${path} documents offline operation`);
  }
});

test("makes one-command cross-platform installers the first quick start", () => {
  assert.match(readme, /install\.ps1/);
  assert.match(readme, /install\.sh/);
  assert.match(readme, /cargo install --git https:\/\/github\.com\/binibinibin123\/geullint/);
  assert.doesNotMatch(readme, /npm install --save-dev geullint/);
});

test("documents CI use, supported targets, and the actual repository", () => {
  assert.match(readme, /geullint \./);
  assert.match(readme, /Windows x64/);
  assert.match(readme, /Windows ARM64/);
  assert.match(readme, /macOS Intel/);
  assert.match(readme, /macOS Apple Silicon/);
  assert.match(readme, /Linux x64/);
  assert.match(readme, /Linux ARM64/);
  assert.match(readme, /binibinibin123\/geullint/);
});

test("keeps archives behind installers and records optional npm publishing", () => {
  assert.match(readme, /fallback|대안/i);
  assert.match(distributionGuide, /NPM_TOKEN/);
  assert.match(distributionGuide, /npm is optional/i);
  assert.match(distributionGuide, /v0\.3\.0-alpha\.1/);
});

test("documents an externally verifiable SBOM attestation command", () => {
  assert.match(distributionGuide, /geullint-v0\.3\.0-alpha\.1-vscode-win32-x64\.vsix/);
  assert.match(
    distributionGuide,
    /--predicate-type\s+https:\/\/spdx\.dev\/Document\/v2\.3/u,
  );
  assert.match(
    distributionGuide,
    /--signer-workflow\s+binibinibin123\/geullint\/\.github\/workflows\/release\.yml/u,
  );
});

test("documents the offline VS Code distribution separately from the CLI", () => {
  assert.match(readme, /VSIX/);
  assert.match(readme, /geullint\.profile/);
  assert.match(readme, /geullint\.userDictionary/);
  assert.match(readme, /geullint\.dictionaryOverlay/);
  assert.match(readme, /geullint\.dictionaryOverlayPaths/);
  assert.match(readme, /geullint\.rulePacks/);
  assert.match(extensionGuide, /geullint\.rulePacks/);
  assert.match(distributionGuide, /여섯 npm/);
  assert.match(distributionGuide, /여섯 플랫폼별 VSIX/);
});

test("documents reproducible release evidence and dictionary attributions", () => {
  assert.match(distributionGuide, /SBOM/);
  assert.match(distributionGuide, /attestation/i);
  assert.match(distributionGuide, /THIRD_PARTY_NOTICES/);
});

test("documents SARIF for code-scanning integrations", () => {
  assert.match(readme, /--format sarif/);
  assert.match(readme, /SARIF/);
});

test("documents the versioned offline dictionary-overlay workflow", () => {
  assert.match(readme, /--dictionary-overlay/);
  assert.match(readme, /dictionary-overlay\.md/);
  assert.match(dictionaryOverlayGuide, /geullint-overlay-v1/);
  assert.match(dictionaryOverlayGuide, /dictionaryOverlayPaths/);
  assert.match(dictionaryOverlayGuide, /\t/);
  assert.match(dictionaryOverlayGuide, /embedded-mecab-ko-dic-v1\.json/);
  assert.match(dictionaryOverlayGuide, /네트워크/);
});

test("documents the versioned offline rule-pack workflow", () => {
  assert.match(readme, /--rule-pack/);
  assert.match(rulePackGuide, /version: 1/);
  assert.match(rulePackGuide, /--corpus/);
  assert.match(rulePackGuide, /네트워크/);
  assert.match(rulePackGuide, /geullint\.rulePacks/);
  assert.doesNotMatch(rulePackGuide, /아직 제공하지 않습니다/);
});

test("documents local corpus metrics and provenance verification", () => {
  assert.match(readme, /--corpus-manifest/);
  assert.match(corpusEvaluationGuide, /--corpus/);
  assert.match(corpusEvaluationGuide, /SHA-256/);
  assert.match(corpusEvaluationGuide, /라이선스/);
  assert.match(corpusEvaluationGuide, /ruleMetrics/);
  assert.match(corpusEvaluationGuide, /macroPrecision/);
  assert.match(corpusEvaluationGuide, /Wilson/);
  assert.match(corpusEvaluationGuide, /--corpus-gate/);
  assert.match(corpusEvaluationGuide, /minMicroPrecision/);
  assert.match(corpusEvaluationGuide, /minExpectedPerRule/);
  assert.match(corpusEvaluationGuide, /requiredRuleIds/);
  assert.match(corpusEvaluationGuide, /네트워크/);
  assert.match(corpusSourcesGuide, /KoLLA/);
  assert.match(corpusSourcesGuide, /GPL-3\.0-or-later/);
  assert.match(corpusSourcesGuide, /kolla-v2-review-queue\.jsonl/);
  assert.match(corpusSourcesGuide, /curate-kolla-v2-gold\.mjs/);
  assert.match(corpusSourcesGuide, /--verify/);
  assert.match(corpusSourcesGuide, /kolla-v2-curated-gold\.provenance\.sha256/);
  assert.match(corpusSourcesGuide, /manifestSha256/);
  assert.match(corpusSourcesGuide, /--require-independent-review/);
  assert.match(corpusSourcesGuide, /independentReviews/);
  assert.match(corpusSourcesGuide, /존재하지 않는 `--out-dir`/);
  assert.match(corpusSourcesGuide, /원자적으로/);
});

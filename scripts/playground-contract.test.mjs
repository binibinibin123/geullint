import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const index = readFileSync("apps/playground/index.html", "utf8");
const app = readFileSync("apps/playground/app.js", "utf8");
const worker = readFileSync("apps/playground/worker.js", "utf8");
const i18n = readFileSync("apps/playground/i18n.js", "utf8");
const buildScript = readFileSync("scripts/build-playground.mjs", "utf8");
const pagesWorkflow = readFileSync(".github/workflows/pages.yml", "utf8");

test("presents the web app as a private Korean spelling checker first", () => {
  assert.match(index, /오픈소스 한국어 맞춤법 검사기/u);
  assert.match(index, /맞춤법·띄어쓰기·문법·문체/u);
  assert.match(i18n, /Open-source Korean spelling and grammar checker/iu);
  assert.match(i18n, /韓国語文章校正・文法チェック/u);
  assert.match(i18n, /韩语拼写与语法检查器/u);
});

test("ships an input, profile selector, and accessible diagnosis output", () => {
  assert.match(index, /id="editor"/);
  assert.match(index, /id="profile"/);
  assert.match(index, /aria-live="polite"/);
  assert.match(index, /모든 검사는 이 브라우저에서만/u);
  assert.match(index, /id="language"/);
  assert.match(index, /id="rule-search"/);
  assert.match(index, /id="rule-list"/);
  assert.match(index, /href="https:\/\/github\.com\/binibinibin123\/geullint\/releases"/);
});

test("ships a separate corrected sentence with copy and apply actions", () => {
  for (const id of [
    "corrected-output",
    "copy-correction",
    "apply-correction",
    "correction-status",
  ]) {
    assert.match(index, new RegExp(`id="${id}"`));
  }
  assert.match(index, /readonly/);
  assert.match(index, /aria-labelledby="correction-heading"/);
  assert.match(app, /data\.response\.fixedText/);
  assert.match(app, /editor\.value\s*!==\s*requestedText/);
  assert.match(app, /navigator\.clipboard\.writeText/);
  assert.match(app, /editor\.value\s*=\s*correctedOutput\.value/);
  for (const key of [
    "correctionTitle",
    "copyCorrection",
    "applyCorrection",
    "correctionApplied",
    "correctionUnchanged",
    "correctionReview",
    "correctionNeedsScan",
    "copySucceeded",
    "copyFailed",
  ]) {
    assert.match(i18n, new RegExp(`${key}:`));
  }
});

test("sends text only to a local Web Worker and never to a network endpoint", () => {
  assert.match(app, /new Worker\("\.\/worker\.js"/);
  assert.match(app, /postMessage/);
  assert.doesNotMatch(app, /fetch\s*\(/);
  assert.doesNotMatch(worker, /fetch\s*\(/);
  assert.match(worker, /lint_json/);
  assert.match(worker, /rule_catalog_json/);
  assert.match(app, /data\.catalog/);
  assert.match(app, /replaceUtf8Range/);
  assert.match(app, /localStorage/);
});

test("ships four interface languages and a searchable curated local catalogue", () => {
  for (const locale of ["ko", "en", "ja", "zh-CN"]) {
    assert.match(i18n, new RegExp(`${JSON.stringify(locale)}:`));
  }
  assert.match(app, /createRuleIndex/);
  assert.match(app, /ruleSearch/);
  assert.match(app, /indexedRules\.length/);
});

test("builds the browser package from the checked Rust WASM artifact", () => {
  assert.match(buildScript, /wasm-bindgen/);
  assert.match(buildScript, /geullint_wasm\.wasm/);
  assert.match(buildScript, /"apps", "playground", "pkg"/);
});

test("deploys the generated static playground through GitHub Pages", () => {
  assert.match(pagesWorkflow, /wasm32-unknown-unknown/);
  assert.match(pagesWorkflow, /wasm-bindgen-cli/);
  assert.match(pagesWorkflow, /actions\/deploy-pages/);
  assert.match(pagesWorkflow, /apps\/playground/);
  assert.match(index, /src="\.\/app\.js"/);
  assert.match(index, /href="\.\/app\.css"/);
});

test("advertises only installation paths that exist at release time", () => {
  assert.match(index, /install\.ps1/u);
  assert.match(index, /install\.sh/u);
  assert.doesNotMatch(index, /npm install --save-dev geullint/u);
  assert.doesNotMatch(index, /npmjs\.com\/package\/geullint/u);
});

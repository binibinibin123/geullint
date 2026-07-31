import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const workflow = readFileSync(".github/workflows/release.yml", "utf8");
const ciWorkflow = readFileSync(".github/workflows/ci.yml", "utf8");
const notices = readFileSync("THIRD_PARTY_NOTICES.md", "utf8");

test("releases only from version tags or an explicit manual run", () => {
  assert.match(workflow, /tags:\s*\["v\*"\]/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /extensions\/vscode-geullint\/package\.json/);
  assert.match(workflow, /quality-report-v\$\{release_version\}\.md/);
});

test("builds every native target and keeps npm publication optional", () => {
  for (const packageName of [
    "geullint-win32-x64",
    "geullint-win32-arm64",
    "geullint-darwin-x64",
    "geullint-darwin-arm64",
    "geullint-linux-x64",
    "geullint-linux-arm64",
  ]) {
    assert.match(workflow, new RegExp(packageName));
  }

  assert.match(workflow, /build-platform-artifacts:/);
  assert.match(workflow, /publish-platform-packages:/);
  assert.match(workflow, /needs:\s*\[validate-version,\s*build-platform-artifacts\]/);
  assert.match(workflow, /HAS_NPM_TOKEN:\s*\$\{\{\s*secrets\.NPM_TOKEN != ''\s*\}\}/);
  assert.match(workflow, /if:\s*env\.HAS_NPM_TOKEN == 'true'/);
  assert.doesNotMatch(workflow, /if:\s*env\.NODE_AUTH_TOKEN != ''/);
  assert.match(workflow, /npm publish --access public/);
  assert.match(workflow, /npm publish --access public --tag next/);
  assert.match(workflow, /NPM_TOKEN/);
  assert.match(workflow, /windows-11-arm/);
  assert.match(workflow, /ubuntu-24\.04-arm/);
});

test("creates archive releases with checksums instead of publishing bare executables", () => {
  assert.match(workflow, /Compress-Archive/);
  assert.match(workflow, /tar -C/);
  assert.match(workflow, /Get-FileHash/);
  assert.match(workflow, /gh release create/);
  assert.match(workflow, /gh release view/);
  assert.match(workflow, /gh release upload[\s\S]*--clobber/u);
  assert.match(workflow, /node scripts\/release-smoke\.mjs/);
  assert.match(workflow, /--prerelease/u);
  assert.match(workflow, /"\$\{RELEASE_VERSION\}" == \*-\*/u);
});

test("packages a platform-matched offline language server inside each VSIX", () => {
  assert.match(workflow, /package-vscode-extension:/);
  assert.match(workflow, /cargo build --release -p geullint-lsp/);
  assert.match(workflow, /extensions\/vscode-geullint\/server/);
  assert.match(workflow, /vsce package --no-dependencies/);
  assert.match(workflow, /npm run compile[\s\S]*vsce package --no-dependencies/u);
  for (const targetName of [
    "win32-x64",
    "win32-arm64",
    "darwin-x64",
    "darwin-arm64",
    "linux-x64",
    "linux-arm64",
  ]) {
    assert.match(workflow, new RegExp(targetName));
  }
  assert.doesNotMatch(workflow, /macos-13|macos-14/);
  assert.match(workflow, /needs: \[validate-version, build-platform-artifacts, package-vscode-extension\]/);
  const releaseJob = workflow.match(
    /\r?\n  create-github-release:\r?\n(?<body>[\s\S]*)$/u,
  )?.groups?.body;
  assert.ok(releaseJob, "release workflow contains the GitHub Release job");
  assert.doesNotMatch(releaseJob, /NPM_TOKEN|publish-platform-packages|publish-launcher/);
});

test("attests release binaries and ships their dependency notices", () => {
  assert.match(workflow, /attestations: write/);
  assert.match(workflow, /id-token: write/);
  assert.match(workflow, /anchore\/sbom-action@v0/);
  assert.match(workflow, /actions\/attest@v4/);
  assert.match(workflow, /sbom-path:/);
  assert.match(workflow, /THIRD_PARTY_NOTICES\.md/);
  assert.match(notices, /Lindera/);
  assert.match(notices, /mecab-ko-dic/);
  assert.match(notices, /Apache License, Version 2\.0/);
  assert.match(notices, /RustCrypto SHA-2/);
  assert.match(notices, /MIT-RustCrypto\.txt/);
});

test("checks the npm launcher and its publishable contents in pull-request CI", () => {
  const rustJob = ciWorkflow.match(
    /\r?\n  rust:\r?\n(?<body>[\s\S]*?)\r?\n  npm-wrapper:/u,
  )?.groups?.body;
  assert.ok(rustJob, "CI contains a Rust job");
  assert.match(rustJob, /actions\/setup-node@v7/);
  assert.match(ciWorkflow, /npm-wrapper:/);
  assert.match(ciWorkflow, /node --test packages\/npm\/geullint\/test\/geullint\.test\.js/);
  assert.match(ciWorkflow, /node --test scripts\/\*\.test\.mjs/);
  assert.match(ciWorkflow, /npm pack --dry-run --json/);
});

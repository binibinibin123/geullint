import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const workflow = readFileSync(".github/workflows/release.yml", "utf8");
const ciWorkflow = readFileSync(".github/workflows/ci.yml", "utf8");
const pagesWorkflow = readFileSync(".github/workflows/pages.yml", "utf8");
const notices = readFileSync("THIRD_PARTY_NOTICES.md", "utf8");

const actionPins = new Map([
  ["actions/checkout", "3d3c42e5aac5ba805825da76410c181273ba90b1"],
  ["dtolnay/rust-toolchain", "2c7215f132e9ebf062739d9130488b56d53c060c"],
  ["actions/setup-node", "820762786026740c76f36085b0efc47a31fe5020"],
  ["actions/configure-pages", "45bfe0192ca1faeb007ade9deae92b16b8254a0d"],
  ["actions/upload-pages-artifact", "fc324d3547104276b827a68afc52ff2a11cc49c9"],
  ["actions/deploy-pages", "cd2ce8fcbc39b97be8ca5fce6e763baed58fa128"],
  ["actions/upload-artifact", "043fb46d1a93c77aae656e7c1c64a875d1fc6a0a"],
  ["actions/download-artifact", "3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c"],
  ["actions/attest", "508db95dd578ae2727ebd6217d5ba78e4fbda05d"],
  ["anchore/sbom-action", "e22c389904149dbc22b58101806040fa8d37a610"],
]);

function jobBody(source, jobName) {
  const marker = `\n  ${jobName}:\n`;
  const start = source.indexOf(marker);
  assert.notEqual(start, -1, `workflow contains the ${jobName} job`);
  const remainder = source.slice(start + marker.length);
  const nextJob = remainder.search(/\n  [a-z][a-z0-9-]*:\n/u);
  return nextJob === -1 ? remainder : remainder.slice(0, nextJob);
}

function actionUses(source) {
  return [...source.matchAll(/uses:\s*([\w.-]+\/[\w.-]+)@([^\s#]+)(?:\s+#\s*(.+))?/gu)].map(
    ([, action, revision, comment]) => ({ action, revision, comment }),
  );
}

test("pins every third-party action to its audited commit with a version comment", () => {
  for (const [name, source] of [
    ["CI", ciWorkflow],
    ["Pages", pagesWorkflow],
    ["Release", workflow],
  ]) {
    const uses = actionUses(source);
    assert.ok(uses.length > 0, `${name} workflow uses actions`);
    for (const { action, revision, comment } of uses) {
      assert.equal(revision, actionPins.get(action), `${name}: ${action} uses the audited SHA`);
      assert.match(comment ?? "", /\S/u, `${name}: ${action} keeps a readable version comment`);
    }
  }
});

test("resolves one immutable release commit and checks it out in every release job", () => {
  const validateJob = jobBody(workflow, "validate-version");
  assert.match(validateJob, /release_ref=refs\/tags\/v\$\{release_version\}/u);
  assert.match(validateJob, /release_sha/u);
  assert.match(validateJob, /git rev-parse "\$\{release_ref\}\^\{commit\}"/u);
  assert.match(
    validateJob,
    /ref:\s*\$\{\{\s*steps\.release_ref\.outputs\.release_ref\s*\}\}/u,
  );

  for (const jobName of [
    "verify-release",
    "build-platform-artifacts",
    "publish-platform-packages",
    "publish-launcher",
    "package-vscode-extension",
    "create-github-release",
  ]) {
    const body = jobBody(workflow, jobName);
    assert.match(body, /uses:\s*actions\/checkout@/u, `${jobName} checks out source`);
    assert.match(
      body,
      /ref:\s*\$\{\{\s*needs\.validate-version\.outputs\.release_sha\s*\}\}/u,
      `${jobName} checks out the validated commit`,
    );
  }
});

test("releases only from version tags or an explicit manual tag selection", () => {
  assert.match(workflow, /tags:\s*\["v\*"\]/u);
  assert.match(workflow, /workflow_dispatch:/u);
  assert.match(workflow, /\[\[ "\$GITHUB_EVENT_NAME" == "workflow_dispatch" \]\]/u);
  assert.match(workflow, /\[\[ ! "\$release_version" =~ \^\[0-9\]\+/u);
  assert.match(workflow, /extensions\/vscode-geullint\/package\.json/u);
  assert.match(workflow, /quality-report-v\$\{release_version\}\.md/u);
  assert.match(
    jobBody(workflow, "validate-version"),
    /"\$GITHUB_EVENT_NAME" == "push"[^\n]+"\$release_sha" != "\$GITHUB_SHA"/u,
    "a moved tag cannot replace the commit that triggered a tag-push release",
  );
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

  assert.match(workflow, /build-platform-artifacts:/u);
  assert.match(workflow, /publish-platform-packages:/u);
  assert.match(workflow, /needs:\s*\[validate-version,\s*verify-release,\s*build-platform-artifacts\]/u);
  assert.match(workflow, /HAS_NPM_TOKEN:\s*\$\{\{\s*secrets\.NPM_TOKEN != ''\s*\}\}/u);
  assert.match(workflow, /if:\s*env\.HAS_NPM_TOKEN == 'true'/u);
  assert.doesNotMatch(workflow, /if:\s*env\.NODE_AUTH_TOKEN != ''/u);
  assert.match(workflow, /npm publish --access public/u);
  assert.match(workflow, /npm publish --access public --tag next/u);
  assert.match(workflow, /NPM_TOKEN/u);
  assert.match(workflow, /windows-11-arm/u);
  assert.match(workflow, /ubuntu-24\.04-arm/u);
  assert.match(
    jobBody(workflow, "build-platform-artifacts"),
    /cargo build --release --locked -p geullint-cli/u,
  );
});

test("restores executable mode on downloaded macOS and Linux npm binaries", () => {
  const publishJob = jobBody(workflow, "publish-platform-packages");
  const downloadIndex = publishJob.indexOf("actions/download-artifact@");
  const chmodIndex = publishJob.indexOf(
    'chmod 0755 "packages/npm/${{ matrix.package_name }}/bin/geullint"',
  );
  const publishIndex = publishJob.indexOf("npm publish --access public");

  assert.match(
    publishJob,
    /if:\s*startsWith\(matrix\.target_name, 'darwin-'\) \|\| startsWith\(matrix\.target_name, 'linux-'\)/u,
  );
  assert.notEqual(chmodIndex, -1, "the downloaded Unix binary is made executable");
  assert.ok(downloadIndex < chmodIndex, "executable mode is restored after artifact download");
  assert.ok(chmodIndex < publishIndex, "executable mode is restored before npm publication");
});

test("includes the project MIT license in every native npm package artifact", () => {
  const buildJob = jobBody(workflow, "build-platform-artifacts");
  const licenseCopyIndex = buildJob.indexOf(
    'Copy-Item LICENSE (Join-Path $packageDirectory "LICENSE")',
  );
  const uploadIndex = buildJob.indexOf("name: npm-${{ matrix.target_name }}");

  assert.notEqual(licenseCopyIndex, -1, "the root MIT license is copied into the package");
  assert.ok(licenseCopyIndex < uploadIndex, "the license is present before the package is uploaded");
});

test("gates every build and publish job on complete release verification", () => {
  const verifyJob = jobBody(workflow, "verify-release");
  for (const command of [
    "cargo fmt --check",
    "cargo clippy --workspace --all-targets --all-features -- -D warnings",
    "cargo test --workspace --all-features",
    "cargo test -p geullint-core --no-default-features",
    "node --test packages/npm/geullint/test/geullint.test.js",
    "npm pack --dry-run --json",
    "node --test scripts/*.test.mjs",
    "npm ci",
    "npm run check",
    "npm test",
    "node scripts/build-playground.mjs",
    "node scripts/wasm-runtime-parity.mjs",
    "node scripts/check-wasm-size.mjs",
    "node scripts/validate-safety-corpus.mjs",
    "--corpus-manifest corpus/safety-regressions-v1.manifest.json",
    "rustup toolchain install 1.88.0",
    "cargo +1.88.0 check --workspace --all-targets --all-features --locked",
  ]) {
    assert.match(verifyJob, new RegExp(command.replaceAll(/[.*+?^${}()|[\]\\]/gu, "\\$&")));
  }

  for (const jobName of [
    "build-platform-artifacts",
    "publish-platform-packages",
    "publish-launcher",
    "package-vscode-extension",
    "create-github-release",
  ]) {
    assert.match(jobBody(workflow, jobName), /needs:\s*\[[^\]]*verify-release[^\]]*\]/u);
  }

  assert.match(ciWorkflow, /cargo build -p geullint-cli/u);
  assert.match(ciWorkflow, /node scripts\/validate-safety-corpus\.mjs/u);
  assert.match(ciWorkflow, /--corpus-manifest corpus\/safety-regressions-v1\.manifest\.json/u);
});

test("creates checksummed archive releases instead of publishing bare executables", () => {
  assert.match(workflow, /Compress-Archive/u);
  assert.match(workflow, /tar -C/u);
  assert.match(workflow, /Get-FileHash/u);
  assert.match(workflow, /gh release create/u);
  assert.match(workflow, /gh release view/u);
  assert.match(workflow, /gh release upload[\s\S]*--clobber/u);
  assert.match(workflow, /node scripts\/release-smoke\.mjs/u);
  assert.match(workflow, /--prerelease/u);
  assert.match(workflow, /"\$\{RELEASE_VERSION\}" == \*-\*/u);
});

test("packages a platform-matched offline language server inside each VSIX", () => {
  assert.match(workflow, /package-vscode-extension:/u);
  assert.match(workflow, /cargo build --release --locked -p geullint-lsp/u);
  assert.match(workflow, /extensions\/vscode-geullint\/server/u);
  assert.match(workflow, /vsce package --no-dependencies/u);
  assert.match(workflow, /npm run build[\s\S]*vsce package --no-dependencies/u);
  assert.match(workflow, /node \.\.\/\.\.\/scripts\/vsix-smoke\.mjs/u);
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
  assert.doesNotMatch(workflow, /macos-13|macos-14/u);
  assert.match(
    workflow,
    /needs: \[validate-version, verify-release, build-platform-artifacts, package-vscode-extension\]/u,
  );
  const releaseJob = jobBody(workflow, "create-github-release");
  assert.doesNotMatch(releaseJob, /NPM_TOKEN|publish-platform-packages|publish-launcher/u);
});

test("attests the archives and VSIX files that users actually download", () => {
  const archiveJob = jobBody(workflow, "build-platform-artifacts");
  assert.match(archiveJob, /file:\s*release-staging\/geullint-v\$\{\{\s*env\.RELEASE_VERSION\s*\}\}/u);
  assert.match(
    archiveJob,
    /subject-path:\s*\$\{\{\s*steps\.archive\.outputs\.archive\s*\}\}/u,
  );
  assert.doesNotMatch(archiveJob, /subject-path:\s*target\/release/u);

  const vscodeJob = jobBody(workflow, "package-vscode-extension");
  assert.match(vscodeJob, /Get-FileHash[\s\S]*"\$vsix\.sha256"/u);
  assert.match(vscodeJob, /file:\s*release-staging\/vsix-/u);
  assert.match(vscodeJob, /subject-path:\s*release-artifacts\/.*\.vsix/u);

  assert.match(workflow, /attestations: write/u);
  assert.match(workflow, /id-token: write/u);
  assert.match(workflow, /sbom-path:/u);
  assert.match(workflow, /THIRD_PARTY_NOTICES\.md/u);
  assert.match(notices, /Lindera/u);
  assert.match(notices, /mecab-ko-dic/u);
  assert.match(notices, /Apache License, Version 2\.0/u);
  assert.match(notices, /RustCrypto SHA-2/u);
  assert.match(notices, /MIT-RustCrypto\.txt/u);
});

test("validates, checksums, and attests both installer scripts", () => {
  const releaseJob = jobBody(workflow, "create-github-release");
  assert.match(releaseJob, /cp install\.sh install\.ps1 release-assets\//u);
  assert.match(releaseJob, /bash -n release-assets\/install\.sh/u);
  assert.match(releaseJob, /Parser\]::ParseFile/u);
  assert.match(releaseJob, /install\.sh\.sha256/u);
  assert.match(releaseJob, /install\.ps1\.sha256/u);
  assert.match(releaseJob, /subject-path:[\s\S]*release-assets\/install\.sh[\s\S]*release-assets\/install\.ps1/u);
});

test("derives release notes from the matching changelog section", () => {
  const releaseJob = jobBody(workflow, "create-github-release");
  assert.match(releaseJob, /CHANGELOG\.md/u);
  assert.match(releaseJob, /release-notes\.md/u);
  assert.match(releaseJob, /--notes-file release-notes\.md/u);
  assert.doesNotMatch(releaseJob, /First public alpha/u);
  assert.doesNotMatch(releaseJob, /release_notes=/u);
});

test("checks the npm launcher and its publishable contents in pull-request CI", () => {
  const rustJob = jobBody(ciWorkflow, "rust");
  assert.match(rustJob, /actions\/setup-node@/u);
  assert.match(ciWorkflow, /npm-wrapper:/u);
  assert.match(ciWorkflow, /node --test packages\/npm\/geullint\/test\/geullint\.test\.js/u);
  assert.match(ciWorkflow, /node --test scripts\/\*\.test\.mjs/u);
  assert.match(ciWorkflow, /npm pack --dry-run --json/u);
});

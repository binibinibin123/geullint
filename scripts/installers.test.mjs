import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const shell = readFileSync("install.sh", "utf8");
const powershell = readFileSync("install.ps1", "utf8");
const release = readFileSync(".github/workflows/release.yml", "utf8");

test("shell installer selects every Unix release target and verifies SHA-256", () => {
  assert.match(shell, /Darwin\) operating_system="darwin"/u);
  assert.match(shell, /Linux\) operating_system="linux"/u);
  assert.match(shell, /x86_64 \| amd64\) architecture="x64"/u);
  assert.match(shell, /arm64 \| aarch64\) architecture="arm64"/u);
  assert.match(shell, /target="\$\{operating_system\}-\$\{architecture\}"/u);
  assert.match(shell, /sha256sum|shasum/u);
  assert.match(shell, /GEULLINT_INSTALL_DIR/u);
  assert.match(shell, /GEULLINT_VERSION/u);
  assert.match(shell, /releases\?per_page=1/u);
  assert.doesNotMatch(shell, /releases\/latest/u);
  assert.doesNotMatch(shell, /\beval\b/u);
});

test("PowerShell installer supports Windows x64 and ARM64 and verifies SHA-256", () => {
  assert.match(powershell, /win32-x64/u);
  assert.match(powershell, /win32-arm64/u);
  assert.match(powershell, /Get-FileHash/u);
  assert.match(powershell, /GEULLINT_INSTALL_DIR/u);
  assert.match(powershell, /GEULLINT_VERSION/u);
  assert.match(powershell, /releases\?per_page=1/u);
  assert.doesNotMatch(powershell, /releases\/latest/u);
  assert.doesNotMatch(powershell, /Invoke-Expression/u);
});

test("release attaches both readable installer scripts", () => {
  assert.match(release, /install\.sh/u);
  assert.match(release, /install\.ps1/u);
  assert.match(release, /one-command installers/iu);
  assert.match(release, /Compress-Archive -Path \$stagingDirectory -DestinationPath \$archive/u);
  assert.doesNotMatch(release, /Compress-Archive -Path "\$stagingDirectory\/\*"/u);
});

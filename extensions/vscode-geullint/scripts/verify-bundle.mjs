import assert from "node:assert/strict";
import { builtinModules } from "node:module";
import { readFileSync } from "node:fs";

const runtimeDependencies = [
  {
    name: "balanced-match",
    version: "4.0.4",
    license: "MIT",
    sourceLicense: "LICENSE.md",
    bundledLicense: "MIT-balanced-match.txt",
  },
  {
    name: "brace-expansion",
    version: "5.0.8",
    license: "MIT",
    sourceLicense: "LICENSE",
    bundledLicense: "MIT-brace-expansion.txt",
  },
  {
    name: "minimatch",
    version: "10.2.5",
    license: "BlueOak-1.0.0",
    sourceLicense: "LICENSE.md",
    bundledLicense: "BlueOak-minimatch.txt",
  },
  {
    name: "semver",
    version: "7.8.5",
    license: "ISC",
    sourceLicense: "LICENSE",
    bundledLicense: "ISC-semver.txt",
  },
  {
    name: "vscode-jsonrpc",
    version: "9.0.1",
    license: "MIT",
    sourceLicense: "License.txt",
    bundledLicense: "MIT-Microsoft-vscode-languageserver-node.txt",
  },
  {
    name: "vscode-languageclient",
    version: "10.1.0",
    license: "MIT",
    sourceLicense: "License.txt",
    bundledLicense: "MIT-Microsoft-vscode-languageserver-node.txt",
  },
  {
    name: "vscode-languageserver-protocol",
    version: "3.18.2",
    license: "MIT",
    sourceLicense: "License.txt",
    bundledLicense: "MIT-Microsoft-vscode-languageserver-node.txt",
  },
  {
    name: "vscode-languageserver-textdocument",
    version: "1.0.13",
    license: "MIT",
    sourceLicense: "License.txt",
    bundledLicense: "MIT-Microsoft-vscode-languageserver-node.txt",
  },
  {
    name: "vscode-languageserver-types",
    version: "3.18.0",
    license: "MIT",
    sourceLicense: "License.txt",
    bundledLicense: "MIT-Microsoft-vscode-languageserver-node.txt",
  },
];

function packageNameForInput(input) {
  const parts = input.replaceAll("\\", "/").split("/");
  if (parts[0] !== "node_modules") return undefined;
  return parts[1].startsWith("@") ? `${parts[1]}/${parts[2]}` : parts[1];
}

const metadata = JSON.parse(readFileSync("dist/extension-meta.json", "utf8"));
const outputEntry = Object.entries(metadata.outputs).find(([path]) =>
  path.replaceAll("\\", "/").endsWith("dist/extension.js"),
);

assert.ok(outputEntry, "the extension bundle output is missing from the esbuild metadata");
assert.ok(
  Object.keys(metadata.inputs).some((path) => path.includes("node_modules/vscode-languageclient/")),
  "vscode-languageclient was not bundled into the extension",
);

const bundledPackages = [
  ...new Set(Object.keys(metadata.inputs).map(packageNameForInput).filter(Boolean)),
].sort();
assert.deepEqual(
  bundledPackages,
  runtimeDependencies.map(({ name }) => name),
  "the VS Code runtime dependency notice contract must match the bundle exactly",
);

const packageLock = JSON.parse(readFileSync("package-lock.json", "utf8"));
const notices = readFileSync("../../THIRD_PARTY_NOTICES.md", "utf8");
for (const dependency of runtimeDependencies) {
  const locked = packageLock.packages[`node_modules/${dependency.name}`];
  assert.equal(locked?.version, dependency.version, `${dependency.name} version notice is stale`);
  assert.equal(locked?.license, dependency.license, `${dependency.name} license notice is stale`);
  assert.ok(
    notices.includes(
      `| \`${dependency.name}\` | \`${dependency.version}\` | ${dependency.license} |`,
    ),
    `${dependency.name} is missing from THIRD_PARTY_NOTICES.md`,
  );
  assert.ok(
    notices.includes(
      `[\`${dependency.bundledLicense}\`](LICENSES/${dependency.bundledLicense})`,
    ),
    `${dependency.bundledLicense} is not linked from THIRD_PARTY_NOTICES.md`,
  );
  assert.equal(
    readFileSync(`../../LICENSES/${dependency.bundledLicense}`, "utf8"),
    readFileSync(`node_modules/${dependency.name}/${dependency.sourceLicense}`, "utf8"),
    `${dependency.bundledLicense} must match the installed package license verbatim`,
  );
}

const builtins = new Set([
  ...builtinModules,
  ...builtinModules.map((module) => `node:${module}`),
]);
for (const imported of outputEntry[1].imports) {
  assert.ok(
    imported.external && (imported.path === "vscode" || builtins.has(imported.path)),
    `unexpected runtime dependency left outside the extension bundle: ${imported.path}`,
  );
}

process.stdout.write("VS Code extension bundle contains its runtime dependencies.\n");

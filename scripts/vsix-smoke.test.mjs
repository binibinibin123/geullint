import assert from "node:assert/strict";
import test from "node:test";
import {
  listVsixEntries,
  validateVsixEntries,
  VSCODE_RUNTIME_NOTICE_ENTRIES,
} from "./vsix-smoke.mjs";

function centralDirectoryFixture(entries) {
  const records = entries.map((entry) => {
    const name = Buffer.from(entry);
    const record = Buffer.alloc(46 + name.length);
    record.writeUInt32LE(0x02014b50, 0);
    record.writeUInt16LE(name.length, 28);
    name.copy(record, 46);
    return record;
  });
  const directory = Buffer.concat(records);
  const footer = Buffer.alloc(22);
  footer.writeUInt32LE(0x06054b50, 0);
  footer.writeUInt16LE(entries.length, 8);
  footer.writeUInt16LE(entries.length, 10);
  footer.writeUInt32LE(directory.length, 12);
  footer.writeUInt32LE(0, 16);
  return Buffer.concat([directory, footer]);
}

test("accepts a bundled platform VSIX with its offline server", () => {
  const entries = [
    "extension/package.json",
    "extension/dist/extension.js",
    "extension/LICENSE.txt",
    ...VSCODE_RUNTIME_NOTICE_ENTRIES,
    "extension/server/win32-x64/geullint-lsp.exe",
  ];
  assert.deepEqual(listVsixEntries(centralDirectoryFixture(entries)), entries);
  assert.doesNotThrow(() => validateVsixEntries(entries, "win32-x64", "geullint-lsp.exe"));
});

test("rejects an extension without its bundled runtime or language server", () => {
  assert.throws(
    () => validateVsixEntries(["extension/package.json"], "linux-x64", "geullint-lsp"),
    /dist\/extension\.js/u,
  );
});

test("rejects a VSIX that accidentally carries a second platform server", () => {
  const entries = [
    "extension/package.json",
    "extension/dist/extension.js",
    "extension/LICENSE.txt",
    ...VSCODE_RUNTIME_NOTICE_ENTRIES,
    "extension/server/linux-x64/geullint-lsp",
    "extension/server/win32-x64/geullint-lsp.exe",
  ];
  assert.throws(
    () => validateVsixEntries(entries, "linux-x64", "geullint-lsp"),
    /exactly one platform-matched language server/u,
  );
});

test("rejects a VSIX without the bundled runtime notices", () => {
  const entries = [
    "extension/package.json",
    "extension/dist/extension.js",
    "extension/LICENSE.txt",
    "extension/server/linux-x64/geullint-lsp",
  ];
  assert.throws(
    () => validateVsixEntries(entries, "linux-x64", "geullint-lsp"),
    /THIRD_PARTY_NOTICES\.md/u,
  );
});

test("rejects a VSIX missing a bundled npm dependency license", () => {
  const entries = [
    "extension/package.json",
    "extension/dist/extension.js",
    "extension/LICENSE.txt",
    "extension/THIRD_PARTY_NOTICES.md",
    "extension/LICENSES/MIT-Microsoft-vscode-languageserver-node.txt",
    "extension/LICENSES/ISC-semver.txt",
    "extension/LICENSES/BlueOak-minimatch.txt",
    "extension/LICENSES/MIT-balanced-match.txt",
    "extension/server/linux-x64/geullint-lsp",
  ];
  assert.throws(
    () => validateVsixEntries(entries, "linux-x64", "geullint-lsp"),
    /MIT-brace-expansion\.txt/u,
  );
});

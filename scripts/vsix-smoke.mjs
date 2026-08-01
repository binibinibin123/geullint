import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const CENTRAL_DIRECTORY_SIGNATURE = 0x02014b50;
const END_OF_CENTRAL_DIRECTORY_SIGNATURE = 0x06054b50;

export const VSCODE_RUNTIME_NOTICE_ENTRIES = [
  "extension/THIRD_PARTY_NOTICES.md",
  "extension/LICENSES/MIT-Microsoft-vscode-languageserver-node.txt",
  "extension/LICENSES/ISC-semver.txt",
  "extension/LICENSES/BlueOak-minimatch.txt",
  "extension/LICENSES/MIT-balanced-match.txt",
  "extension/LICENSES/MIT-brace-expansion.txt",
];

export function listVsixEntries(archive) {
  const minimumEndOffset = Math.max(0, archive.length - 65_557);
  let endOffset = archive.length - 22;
  while (
    endOffset >= minimumEndOffset
    && archive.readUInt32LE(endOffset) !== END_OF_CENTRAL_DIRECTORY_SIGNATURE
  ) {
    endOffset -= 1;
  }
  assert.ok(endOffset >= minimumEndOffset, "VSIX central directory footer is missing");

  const entryCount = archive.readUInt16LE(endOffset + 10);
  let offset = archive.readUInt32LE(endOffset + 16);
  const entries = [];
  for (let index = 0; index < entryCount; index += 1) {
    assert.equal(
      archive.readUInt32LE(offset),
      CENTRAL_DIRECTORY_SIGNATURE,
      "VSIX central directory entry is malformed",
    );
    const nameLength = archive.readUInt16LE(offset + 28);
    const extraLength = archive.readUInt16LE(offset + 30);
    const commentLength = archive.readUInt16LE(offset + 32);
    entries.push(archive.subarray(offset + 46, offset + 46 + nameLength).toString("utf8"));
    offset += 46 + nameLength + extraLength + commentLength;
  }
  return entries;
}

export function validateVsixEntries(entries, targetName, binaryName) {
  const files = new Set(entries);
  for (const required of [
    "extension/package.json",
    "extension/dist/extension.js",
    "extension/LICENSE.txt",
    ...VSCODE_RUNTIME_NOTICE_ENTRIES,
    `extension/server/${targetName}/${binaryName}`,
  ]) {
    assert.ok(files.has(required), `VSIX is missing ${required}`);
  }
  const bundledServers = [...files].filter(
    (path) => path.startsWith("extension/server/") && !path.endsWith("/"),
  );
  assert.deepEqual(
    bundledServers,
    [`extension/server/${targetName}/${binaryName}`],
    "VSIX must contain exactly one platform-matched language server",
  );
  assert.ok(
    [...files].every((path) => !path.startsWith("extension/node_modules/")),
    "bundled VSIX must not carry a loose node_modules tree",
  );
}

function main() {
  const [vsixPath, targetName, binaryName] = process.argv.slice(2);
  assert.ok(vsixPath && targetName && binaryName, "usage: vsix-smoke.mjs VSIX TARGET BINARY");
  const entries = listVsixEntries(readFileSync(vsixPath));
  validateVsixEntries(entries, targetName, binaryName);
  process.stdout.write(`VSIX smoke test passed: ${vsixPath}\n`);
}

if (import.meta.url === pathToFileURL(process.argv[1] || "").href) {
  main();
}

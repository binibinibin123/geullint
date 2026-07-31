import assert from "node:assert/strict";
import test from "node:test";
import {
  parseChecksum,
  validateArchiveEntries,
  validateReleaseCatalog,
} from "./release-smoke.mjs";

const digest = "a".repeat(64);

test("parses the release checksum format and exact archive name", () => {
  assert.equal(
    parseChecksum(`${digest}  geullint-v0.1.0-win32-x64.zip`, "geullint-v0.1.0-win32-x64.zip"),
    digest,
  );
  assert.throws(
    () => parseChecksum(`${digest}  other.zip`, "release.zip"),
    /archive name/,
  );
  assert.throws(
    () => parseChecksum("not-a-checksum", "release.zip"),
    /checksum/,
  );
});

test("requires one executable and the complete license payload", () => {
  const entries = [
    "geullint-v0.1.0-win32-x64/geullint.exe",
    "geullint-v0.1.0-win32-x64/LICENSE",
    "geullint-v0.1.0-win32-x64/NOTICE",
    "geullint-v0.1.0-win32-x64/LICENSES/Apache-2.0.txt",
  ];

  assert.equal(
    validateArchiveEntries(entries, "geullint.exe"),
    "geullint-v0.1.0-win32-x64/geullint.exe",
  );
  assert.throws(
    () =>
      validateArchiveEntries(
        [...entries, "geullint-v0.1.0-win32-x64/other/geullint.exe"],
        "geullint.exe",
      ),
    /exactly one executable/,
  );
  assert.throws(
    () => validateArchiveEntries(entries.filter((entry) => !entry.endsWith("NOTICE")), "geullint.exe"),
    /NOTICE/,
  );
});

test("requires one versioned archive root so installers can locate the binary", () => {
  const flatEntries = [
    "geullint.exe",
    "LICENSE",
    "NOTICE",
    "LICENSES/Apache-2.0.txt",
  ];

  assert.throws(
    () => validateArchiveEntries(flatEntries, "geullint.exe"),
    /versioned root directory/u,
  );
});

test("accepts only a nonempty curated catalogue of at most 100 rules", () => {
  assert.doesNotThrow(() =>
    validateReleaseCatalog({
      ruleCount: 100,
      rules: Array.from({ length: 100 }, (_, index) => ({ id: `rule-${index}` })),
    }),
  );
  assert.throws(
    () => validateReleaseCatalog({ ruleCount: 101, rules: Array(101).fill({}) }),
    /at most 100/u,
  );
  assert.throws(
    () => validateReleaseCatalog({ ruleCount: 2, rules: [{}] }),
    /match/u,
  );
  assert.throws(
    () => validateReleaseCatalog({ ruleCount: 0, rules: [] }),
    /nonempty/u,
  );
});

import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const snapshotPath = "dictionaries/embedded-mecab-ko-dic-v1.json";

test("pins the embedded Korean morphology dictionary to checked dependency checksums", () => {
  assert.ok(existsSync(snapshotPath), "embedded dictionary snapshot provenance exists");

  const snapshot = JSON.parse(readFileSync(snapshotPath, "utf8"));
  const cargoLock = readFileSync("Cargo.lock", "utf8");
  const notices = readFileSync("THIRD_PARTY_NOTICES.md", "utf8");

  assert.equal(snapshot.schemaVersion, 1);
  assert.equal(snapshot.dictionary.package, "lindera-ko-dic");
  assert.equal(snapshot.dictionary.version, "4.0.1");
  assert.equal(snapshot.dictionary.crateSha256, "f52b0d8ee301729759af2dcb04246b3eb5fa1491aa235369156f034aaf709d32");
  assert.equal(snapshot.source.version, "mecab-ko-dic-2.1.1-20180720");
  assert.equal(snapshot.source.license, "Apache-2.0");
  assert.match(cargoLock, /name = "lindera-ko-dic"\r?\nversion = "4\.0\.1"/);
  assert.match(cargoLock, new RegExp(snapshot.dictionary.crateSha256));
  assert.match(notices, new RegExp(snapshot.source.version));
  assert.match(notices, /Apache License, Version 2\.0/);
});

import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

import {
  buildLexicon,
  parseLexiconRows,
  serializeLexicon,
} from "./build-standard-lexicon.mjs";

test("builds a deterministic sorted lexicon and manifest payload", () => {
  const rows = parseLexiconRows([
    "surface\tpos\tfrequency",
    "며칠\tNNG\t120",
    "가다\tVV\t1000",
    "며칠\tNNG\t12",
  ].join("\n"));
  assert.deepEqual(rows, [
    { surface: "가다", pos: "VV", frequency: 1000 },
    { surface: "며칠", pos: "NNG", frequency: 132 },
  ]);
  const bytes = serializeLexicon(rows);
  assert.equal(bytes.toString("utf8"), "geullint-standard-lexicon-v1\n가다\tVV\t1000\n며칠\tNNG\t132\n");
  const result = buildLexicon(rows, { name: "test" });
  assert.equal(result.manifest.schemaVersion, 1);
  assert.equal(result.manifest.entryCount, 2);
  assert.equal(result.manifest.name, "test");
  assert.equal(result.manifest.sha256.length, 64);
  assert.equal(result.manifest.gzipSha256.length, 64);
  assert.equal(result.gzip[9], 3, "gzip metadata must be portable across release hosts");
});

test("rejects malformed rows, unsafe surfaces, and invalid frequencies", () => {
  assert.throws(() => parseLexiconRows("surface\tpos\tfrequency\n\tNNG\t1"), /surface must be non-empty/);
  assert.throws(() => parseLexiconRows("surface\tpos\tfrequency\n가\tNNG\t-1"), /frequency must be a non-negative integer/);
  assert.throws(() => parseLexiconRows("surface\tpos\tfrequency\n가\tNNG\t1\n가\tVV\t1"), /conflicting POS/);
});

test("checked-in standard manifest matches the reproducible seed asset", async () => {
  const seed = await readFile("dictionaries/standard-ko-v1.seed.tsv", "utf8");
  const manifest = JSON.parse(await readFile("dictionaries/standard-ko-v1.manifest.json", "utf8"));
  const result = buildLexicon(parseLexiconRows(seed), { name: manifest.name, version: manifest.version });
  assert.deepEqual(
    Object.fromEntries(Object.keys(result.manifest).map((key) => [key, result.manifest[key]])),
    Object.fromEntries(Object.keys(result.manifest).map((key) => [key, manifest[key]])),
  );
});

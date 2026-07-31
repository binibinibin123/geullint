import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import {
  parseSeed,
  renderCatalog,
  romanizeKorean,
  validateSourceIsolation,
} from "./build-rule-catalog.mjs";

const curatedSeed = "rules/seeds/curated-core.tsv";

test("renders the reviewed lexical core deterministically", () => {
  const source = readFileSync(curatedSeed, "utf8");
  const rules = parseSeed(source);

  assert.equal(rules.length, 42);
  assert.equal(new Set(rules.map((rule) => rule.id)).size, rules.length);
  assert.equal(new Set(rules.map((rule) => rule.from)).size, rules.length);
  assert.ok(rules.every((rule) => rule.from !== rule.to));
  assert.ok(rules.every((rule) => rule.family === "spelling.lexical"));
  assert.equal(renderCatalog(rules), renderCatalog(parseSeed(source)));
  assert.doesNotMatch(renderCatalog(rules), /’을 ‘|’를 ‘/u);
});

test("derives stable ASCII slugs from Korean and Latin terms", () => {
  assert.equal(romanizeKorean("데이터베이스"), "deiteobeiseu");
  assert.equal(romanizeKorean("GitHub"), "github");
  assert.match(romanizeKorean("문장 부호"), /^[a-z0-9-]+$/u);
});

test("rejects duplicate source matchers and malformed seed rows", () => {
  assert.throws(
    () =>
      parseSeed(
        [
          "geullint-rule-seed-v1",
          "spacing.compound\t데이터 베이스\t데이터베이스\thigh\tdefault",
          "spacing.compound\t데이터 베이스\t자료기지\thigh\tdefault",
        ].join("\n"),
      ),
    /duplicate source/u,
  );
  assert.throws(
    () => parseSeed("geullint-rule-seed-v1\nspacing.compound\t같음\t같음\thigh\tdefault"),
    /must differ/u,
  );
});

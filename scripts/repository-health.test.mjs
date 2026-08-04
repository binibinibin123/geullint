import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const requiredMarkdown = [
  "ARCHITECTURE.md",
  "CHANGELOG.md",
  "CODE_OF_CONDUCT.md",
  "CONTRIBUTING.md",
  "ROADMAP.md",
  "SECURITY.md",
  ".github/PULL_REQUEST_TEMPLATE.md",
];

test("ships the standard open-source community documents", () => {
  for (const path of requiredMarkdown) {
    const source = readFileSync(path, "utf8");
    assert.match(source, /^#\s+\S+/u, `${path} has a title`);
    assert.ok(source.length > 300, `${path} contains actionable guidance`);
  }
});

test("ships structured bug, feature, and rule issue forms", () => {
  for (const name of ["bug.yml", "feature.yml", "rule.yml"]) {
    const path = `.github/ISSUE_TEMPLATE/${name}`;
    const source = readFileSync(path, "utf8");
    assert.match(source, /^name:\s*.+$/mu, `${path} has a name`);
    assert.match(source, /^body:\s*$/mu, `${path} has a body`);
    assert.ok(
      [...source.matchAll(/^\s*-\s+type:\s*\S+/gmu)].length >= 4,
      `${path} has useful fields`,
    );
  }
  const config = readFileSync(".github/ISSUE_TEMPLATE/config.yml", "utf8");
  assert.match(config, /^blank_issues_enabled:\s*false$/mu);
  assert.match(config, /SECURITY\.md/u);
});

test("ships machine-readable citation metadata for the curated alpha", () => {
  const citation = readFileSync("CITATION.cff", "utf8");
  assert.match(citation, /^cff-version:\s*1\.2\.0$/mu);
  assert.match(citation, /^version:\s*0\.3\.0-alpha\.2$/mu);
  assert.match(citation, /^license:\s*MIT$/mu);
  assert.match(
    citation,
    /^repository:\s*"https:\/\/github\.com\/binibinibin123\/geullint"$/mu,
  );
});

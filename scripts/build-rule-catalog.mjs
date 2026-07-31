import { readFile, writeFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";

const INITIAL = [
  "g", "kk", "n", "d", "tt", "r", "m", "b", "pp", "s",
  "ss", "", "j", "jj", "ch", "k", "t", "p", "h",
];
const MEDIAL = [
  "a", "ae", "ya", "yae", "eo", "e", "yeo", "ye", "o", "wa",
  "wae", "oe", "yo", "u", "wo", "we", "wi", "yu", "eu", "ui", "i",
];
const FINAL = [
  "", "k", "k", "ks", "n", "nj", "nh", "t", "l", "lk",
  "lm", "lb", "ls", "lt", "lp", "lh", "m", "p", "ps", "t",
  "t", "ng", "t", "t", "k", "t", "p", "h",
];
const VALID_CONFIDENCE = new Set(["high", "medium", "low"]);
const VALID_PROFILE = new Set(["default", "strict", "editorial"]);

export function romanizeKorean(value) {
  const pieces = [];
  for (const character of value.normalize("NFKC")) {
    const codePoint = character.codePointAt(0);
    if (codePoint >= 0xac00 && codePoint <= 0xd7a3) {
      const offset = codePoint - 0xac00;
      const initial = Math.floor(offset / 588);
      const medial = Math.floor((offset % 588) / 28);
      const final = offset % 28;
      pieces.push(`${INITIAL[initial]}${MEDIAL[medial]}${FINAL[final]}`);
    } else if (/[\p{Letter}\p{Number}]/u.test(character)) {
      pieces.push(character.toLowerCase());
    } else {
      pieces.push("-");
    }
  }
  return pieces
    .join("")
    .replace(/[^a-z0-9]+/gu, "-")
    .replace(/^-+|-+$/gu, "")
    .replace(/-{2,}/gu, "-");
}

export function parseSeed(source) {
  const lines = source.replace(/^\uFEFF/u, "").split(/\r?\n/u);
  if (lines.shift() !== "geullint-rule-seed-v1") {
    throw new Error("rule seed must start with geullint-rule-seed-v1");
  }

  const rules = [];
  const ids = new Set();
  const sources = new Set();
  for (const [index, line] of lines.entries()) {
    if (!line || line.startsWith("#")) {
      continue;
    }
    const fields = line.split("\t");
    if (fields.length < 5 || fields.length > 6) {
      throw new Error(`line ${index + 2} must contain five or six tab-separated fields`);
    }
    const [family, from, to, confidence, profile, explicitSlug] = fields;
    if (!/^[a-z][a-z0-9-]*(?:\.[a-z][a-z0-9-]*)+$/u.test(family)) {
      throw new Error(`line ${index + 2} has an invalid rule family`);
    }
    if (!from || !to || from === to) {
      throw new Error(`line ${index + 2} source and correction must differ`);
    }
    if (!VALID_CONFIDENCE.has(confidence) || !VALID_PROFILE.has(profile)) {
      throw new Error(`line ${index + 2} has invalid confidence or profile`);
    }
    if (sources.has(from)) {
      throw new Error(`line ${index + 2} has duplicate source ${JSON.stringify(from)}`);
    }
    sources.add(from);

    const slug = explicitSlug || romanizeKorean(to);
    if (!slug) {
      throw new Error(`line ${index + 2} needs an explicit ASCII slug`);
    }
    const id = `${family}.${slug}`;
    if (ids.has(id)) {
      throw new Error(`line ${index + 2} has duplicate rule ID ${id}`);
    }
    ids.add(id);
    rules.push({ id, family, from, to, confidence, profile });
  }
  return rules;
}

export function validateSourceIsolation(rules) {
  const overlaps = [];
  for (const [index, rule] of rules.entries()) {
    for (const other of rules.slice(index + 1)) {
      if (rule.from.includes(other.from) || other.from.includes(rule.from)) {
        overlaps.push(`${JSON.stringify(rule.from)} and ${JSON.stringify(other.from)}`);
      }
    }
  }
  if (overlaps.length > 0) {
    throw new Error(`overlapping sources: ${overlaps.join("; ")}`);
  }
}

function severityFor(rule) {
  if (rule.family.startsWith("spelling.")) {
    return "error";
  }
  if (rule.family.startsWith("style.") || rule.family.startsWith("advanced.")) {
    return "info";
  }
  return "warning";
}

export function renderCatalog(rules) {
  const lines = ["version: 1", "language: ko", "rules:"];
  for (const rule of rules) {
    const safe = rule.confidence === "high" && rule.profile === "default";
    const title = `${rule.to} 표기`;
    const description = `권장 표기: ‘${rule.to}’`;
    lines.push(`  - id: ${rule.id}`);
    lines.push(`    title: ${JSON.stringify(title)}`);
    lines.push(`    description: ${JSON.stringify(description)}`);
    lines.push(`    severity: ${severityFor(rule)}`);
    lines.push(`    confidence: ${rule.confidence}`);
    if (rule.profile !== "default") {
      lines.push(`    profile: ${rule.profile}`);
    }
    lines.push(`    defaultEnabled: ${rule.profile === "default"}`);
    lines.push(`    message: ${JSON.stringify(description)}`);
    lines.push(`    safeFix: ${safe}`);
    lines.push("    replacements:");
    lines.push(`      - from: ${JSON.stringify(rule.from)}`);
    lines.push(`        to: ${JSON.stringify(rule.to)}`);
    lines.push("    examples:");
    lines.push("      incorrect:");
    lines.push(`        - ${JSON.stringify(rule.from)}`);
    lines.push("      correct:");
    lines.push(`        - ${JSON.stringify(rule.to)}`);
  }
  return `${lines.join("\n")}\n`;
}

async function main(arguments_) {
  if (arguments_.length !== 2) {
    throw new Error("Usage: node scripts/build-rule-catalog.mjs SEED.tsv OUTPUT.yaml");
  }
  const [seedPath, outputPath] = arguments_;
  const rules = parseSeed(await readFile(seedPath, "utf8"));
  await writeFile(outputPath, renderCatalog(rules), "utf8");
  process.stdout.write(`generated ${rules.length} rules at ${outputPath}\n`);
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  main(process.argv.slice(2)).catch((error) => {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 2;
  });
}

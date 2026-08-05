#!/usr/bin/env node
import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ALLOWED_SPLITS = new Set(["train", "dev", "release_holdout", "H1", "H2"]);

function isNonblankString(value) {
  return typeof value === "string" && value.trim().length > 0;
}

function hashBucket(seed, key) {
  const digest = createHash("sha256").update(`${seed}\0${key}`).digest();
  return digest.readUInt32BE(0) / 0x1_0000_0000;
}

function assignedSplit(seed, key) {
  const bucket = hashBucket(seed, key);
  if (bucket < 0.1) return "H1";
  if (bucket < 0.2) return "H2";
  if (bucket < 0.3) return "dev";
  return "train";
}

function provenanceGroup(entry) {
  if (isNonblankString(entry.authorId)) return `author:${entry.authorId.trim()}`;
  if (isNonblankString(entry.documentId)) return `document:${entry.documentId.trim()}`;
  return `case:${entry.id}`;
}

export function splitCorpusByDocument(cases, options = {}) {
  if (!Array.isArray(cases) || cases.length === 0) throw new TypeError("cases must be a non-empty array");
  const seed = options.seed ?? "geullint-case-v2";
  if (!isNonblankString(seed)) throw new TypeError("seed must be a non-empty string");
  const ids = new Set();
  const groups = new Map();
  for (const entry of cases) {
    if (!entry || typeof entry !== "object" || !isNonblankString(entry.id)) throw new Error("case id must be non-empty");
    if (ids.has(entry.id)) throw new Error(`duplicate case id: ${entry.id}`);
    ids.add(entry.id);
    if (typeof entry.text !== "string" || entry.text.length === 0) throw new Error(`case ${entry.id} text must be non-empty`);
    if (entry.textOrigin !== "project") {
      if (!isNonblankString(entry.documentId)) throw new Error(`case ${entry.id} requires documentId`);
      if (!isNonblankString(entry.authorId)) throw new Error(`case ${entry.id} requires authorId`);
    }
    if (entry.split !== undefined && !ALLOWED_SPLITS.has(entry.split)) throw new Error(`case ${entry.id} has unknown split`);
    const key = provenanceGroup(entry);
    const group = groups.get(key) ?? { key, entries: [], splits: new Set() };
    group.entries.push(entry);
    if (entry.split) group.splits.add(entry.split);
    groups.set(key, group);
  }

  for (const group of groups.values()) {
    if (group.splits.size > 1) throw new Error(`author appears in multiple splits: ${group.key}`);
    group.split = group.splits.values().next().value ?? assignedSplit(seed, group.key);
  }

  const output = [];
  for (const group of groups.values()) {
    for (const entry of group.entries) {
      const holdoutId = group.split === "H1" || group.split === "H2" ? group.split : null;
      output.push({ ...entry, split: group.split, holdoutId });
    }
  }
  return output.sort((left, right) => left.id.localeCompare(right.id));
}

async function readJsonLines(path) {
  const contents = await readFile(resolve(path), "utf8");
  return contents.split(/\r?\n/u).filter((line) => line.trim()).map((line, index) => {
    try {
      return JSON.parse(line);
    } catch (error) {
      throw new Error(`${path}:${index + 1} is not valid JSON: ${error.message}`);
    }
  });
}

function argument(arguments_, name) {
  const index = arguments_.indexOf(name);
  const value = index === -1 ? undefined : arguments_[index + 1];
  if (!value || value.startsWith("--")) throw new Error(`missing ${name}`);
  return value;
}

async function main(arguments_) {
  const input = argument(arguments_, "--input");
  const output = argument(arguments_, "--out");
  const seedIndex = arguments_.indexOf("--seed");
  const seed = seedIndex >= 0 ? arguments_[seedIndex + 1] : undefined;
  const split = splitCorpusByDocument(await readJsonLines(input), { seed });
  await writeFile(resolve(output), `${split.map(JSON.stringify).join("\n")}\n`, "utf8");
  process.stdout.write(`${JSON.stringify({ cases: split.length, output: resolve(output) })}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(`geullint: ${error.message}`);
    process.exitCode = 2;
  });
}

#!/usr/bin/env node
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { mergeBlindReviews } from "./ai-adjudication.mjs";

function parseJsonLines(contents, label) {
  return contents.split(/\r?\n/u).filter((line) => line.trim()).map((line, index) => {
    try {
      return JSON.parse(line);
    } catch (error) {
      throw new Error(`${label}:${index + 1} is not valid JSON: ${error.message}`);
    }
  });
}

function groupPackets(records, label) {
  const groups = new Map();
  for (const record of records) {
    if (!record || typeof record !== "object" || typeof record.caseId !== "string" || !record.caseId.trim()) {
      throw new Error(`${label} records require a non-empty caseId`);
    }
    if (!groups.has(record.caseId)) groups.set(record.caseId, []);
    const packet = { ...record };
    delete packet.caseId;
    groups.get(record.caseId).push(packet);
  }
  return groups;
}

export function mergeReviewPackets(baseCases, reviewRecords, adjudicationRecords = []) {
  if (!Array.isArray(baseCases) || !Array.isArray(reviewRecords) || !Array.isArray(adjudicationRecords)) {
    throw new TypeError("baseCases, reviewRecords, and adjudicationRecords must be arrays");
  }
  const baseIds = new Set();
  for (const entry of baseCases) {
    if (!entry || typeof entry.id !== "string" || !entry.id.trim()) throw new Error("base cases require non-empty IDs");
    if (baseIds.has(entry.id)) throw new Error(`duplicate base case ID: ${entry.id}`);
    baseIds.add(entry.id);
  }
  const reviewsByCase = groupPackets(reviewRecords, "review");
  const adjudicationsByCase = groupPackets(adjudicationRecords, "adjudication");
  for (const [caseId, packets] of reviewsByCase) {
    const reviewerIds = new Set();
    for (const packet of packets) {
      if (reviewerIds.has(packet.reviewerId)) throw new Error(`duplicate review packet for ${caseId}: ${packet.reviewerId}`);
      reviewerIds.add(packet.reviewerId);
    }
    if (!baseIds.has(caseId)) throw new Error(`review packet references unknown case: ${caseId}`);
  }
  for (const [caseId, packets] of adjudicationsByCase) {
    if (packets.length !== 1) throw new Error(`case ${caseId} must have one adjudication packet`);
    if (!baseIds.has(caseId)) throw new Error(`adjudication references unknown case: ${caseId}`);
  }
  return baseCases.map((base) => {
    const reviews = reviewsByCase.get(base.id) ?? [];
    const adjudications = adjudicationsByCase.get(base.id) ?? [];
    if (reviews.length < 2) throw new Error(`case ${base.id} requires at least two review packets`);
    return mergeBlindReviews(base, reviews, adjudications[0]);
  });
}

async function readJsonLines(path) {
  return parseJsonLines(await readFile(resolve(path), "utf8"), path);
}

function argument(arguments_, name, required = true) {
  const index = arguments_.indexOf(name);
  const value = index === -1 ? undefined : arguments_[index + 1];
  if (required && (!value || value.startsWith("--"))) throw new Error(`missing ${name}`);
  return value && !value.startsWith("--") ? value : undefined;
}

async function main(arguments_) {
  const casesPath = argument(arguments_, "--cases");
  const reviewsPath = argument(arguments_, "--reviews");
  const adjudicationsPath = argument(arguments_, "--adjudications", false);
  const outputPath = argument(arguments_, "--out");
  const [baseCases, reviews, adjudications] = await Promise.all([
    readJsonLines(casesPath),
    readJsonLines(reviewsPath),
    adjudicationsPath ? readJsonLines(adjudicationsPath) : Promise.resolve([]),
  ]);
  const merged = mergeReviewPackets(baseCases, reviews, adjudications);
  await writeFile(resolve(outputPath), `${merged.map(JSON.stringify).join("\n")}\n`, "utf8");
  process.stdout.write(`${JSON.stringify({ cases: merged.length, output: resolve(outputPath) })}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(`geullint: ${error.message}`);
    process.exitCode = 2;
  });
}

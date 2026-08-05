#!/usr/bin/env node
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const HASH_FIELDS = ["rubricSha256", "sessionSha256", "outputSha256"];
const HASH = /^[0-9a-f]{64}$/u;
const DEFAULTS = {
  minCases: 1,
  minReviewers: 2,
  minAgreementRate: 0,
  maxAdjudicationRate: 1,
  maxAuditDisagreementRate: 1,
  maxMissingProvenanceHashes: 0,
  requireAdjudicationForConflicts: true,
  requireAiReviewers: true,
};

function isNonblankString(value) {
  return typeof value === "string" && value.trim().length > 0;
}

function normalizeDiagnostics(diagnostics) {
  if (!Array.isArray(diagnostics)) return [];
  return diagnostics
    .map((diagnostic) => ({
      ruleId: diagnostic?.ruleId ?? "",
      range: diagnostic?.range ?? null,
      suggestions: Array.isArray(diagnostic?.suggestions) ? [...diagnostic.suggestions].sort() : [],
    }))
    .sort((left, right) => JSON.stringify(left).localeCompare(JSON.stringify(right)));
}

function signature(packet) {
  return JSON.stringify({
    status: packet.status,
    diagnostics: normalizeDiagnostics(packet.diagnostics),
  });
}

function addFailure(failures, metric, actual, expected, caseId = undefined) {
  failures.push({
    metric,
    actual,
    ...(expected !== undefined ? { expected } : {}),
    ...(caseId ? { caseId } : {}),
  });
}

function validPacket(packet) {
  return packet && typeof packet === "object" && !Array.isArray(packet)
    && isNonblankString(packet.caseId)
    && isNonblankString(packet.reviewerId);
}

export function evaluateReviewQuality(reviews, adjudications = [], gate = {}) {
  if (!Array.isArray(reviews) || !Array.isArray(adjudications)) {
    throw new TypeError("reviews and adjudications must be arrays");
  }
  const thresholds = { ...DEFAULTS, ...(gate.reviewQuality ?? gate) };
  const failures = [];
  const groups = new Map();
  const adjudicationByCase = new Map();
  const reviewers = new Set();
  let missingProvenanceHashes = 0;
  let nonAiReviewers = 0;
  let duplicateReviewerPackets = 0;

  const registerPacket = (packet, target, label) => {
    if (!validPacket(packet)) {
      addFailure(failures, "invalidPacket", 1, 0, packet?.caseId);
      return;
    }
    if (packet.reviewerType !== "ai") nonAiReviewers += 1;
    if (label === "review") reviewers.add(packet.reviewerId);
    for (const field of HASH_FIELDS) {
      if (!HASH.test(packet[field] ?? "")) missingProvenanceHashes += 1;
    }
    const list = target.get(packet.caseId) ?? [];
    if (label === "review") {
      if (list.some((entry) => entry.reviewerId === packet.reviewerId)) duplicateReviewerPackets += 1;
    } else if (list.length > 0) {
      duplicateReviewerPackets += 1;
    }
    list.push(packet);
    target.set(packet.caseId, list);
  };

  for (const packet of reviews) registerPacket(packet, groups, "review");
  for (const packet of adjudications) registerPacket(packet, adjudicationByCase, "adjudication");

  const caseIds = new Set([...groups.keys(), ...adjudicationByCase.keys()]);
  let unanimousCases = 0;
  let conflictCases = 0;
  let adjudicatedCases = 0;
  let missingAdjudication = 0;
  let auditDisagreements = 0;
  const statusCounts = {};

  for (const caseId of caseIds) {
    const packets = groups.get(caseId) ?? [];
    const signatures = new Map();
    for (const packet of packets) {
      const key = signature(packet);
      signatures.set(key, (signatures.get(key) ?? 0) + 1);
      statusCounts[packet.status] = (statusCounts[packet.status] ?? 0) + 1;
    }
    if (packets.length < thresholds.minReviewers) {
      addFailure(failures, "reviewersPerCase", packets.length, thresholds.minReviewers, caseId);
    }
    const conflict = signatures.size > 1;
    if (conflict) {
      conflictCases += 1;
      const adjudicationsForCase = adjudicationByCase.get(caseId) ?? [];
      if (adjudicationsForCase.length === 0) {
        missingAdjudication += 1;
      } else {
        adjudicatedCases += 1;
        const adjudication = adjudicationsForCase[0];
        const reviewerIds = new Set(packets.map((packet) => packet.reviewerId));
        if (reviewerIds.has(adjudication.reviewerId)) duplicateReviewerPackets += 1;
        const majority = [...signatures.entries()].sort((left, right) => right[1] - left[1])[0]?.[0];
        if (signature(adjudication) !== majority) auditDisagreements += 1;
      }
    } else {
      unanimousCases += 1;
    }
  }

  const cases = groups.size;
  const agreementRate = cases === 0 ? null : unanimousCases / cases;
  const adjudicationRate = cases === 0 ? null : conflictCases / cases;
  const auditDisagreementRate = adjudicatedCases === 0 ? 0 : auditDisagreements / adjudicatedCases;
  const metrics = {
    cases,
    reviewPackets: reviews.length,
    reviewers: reviewers.size,
    unanimousCases,
    conflictCases,
    adjudicatedCases,
    missingAdjudication,
    agreementRate,
    adjudicationRate,
    auditDisagreements,
    auditDisagreementRate,
    missingProvenanceHashes,
    nonAiReviewers,
    duplicateReviewerPackets,
    statusCounts,
  };

  if (cases < thresholds.minCases) addFailure(failures, "cases", cases, thresholds.minCases);
  if (reviewers.size < thresholds.minReviewers) addFailure(failures, "reviewers", reviewers.size, thresholds.minReviewers);
  if (thresholds.requireAiReviewers && nonAiReviewers > 0) addFailure(failures, "nonAiReviewers", nonAiReviewers, 0);
  if (duplicateReviewerPackets > 0) addFailure(failures, "duplicateReviewerPackets", duplicateReviewerPackets, 0);
  if (missingProvenanceHashes > thresholds.maxMissingProvenanceHashes) {
    addFailure(failures, "missingProvenanceHashes", missingProvenanceHashes, thresholds.maxMissingProvenanceHashes);
  }
  if (agreementRate === null || agreementRate < thresholds.minAgreementRate) {
    addFailure(failures, "agreementRate", agreementRate, thresholds.minAgreementRate);
  }
  if (adjudicationRate !== null && adjudicationRate > thresholds.maxAdjudicationRate) {
    addFailure(failures, "adjudicationRate", adjudicationRate, thresholds.maxAdjudicationRate);
  }
  if (auditDisagreementRate > thresholds.maxAuditDisagreementRate) {
    addFailure(failures, "auditDisagreementRate", auditDisagreementRate, thresholds.maxAuditDisagreementRate);
  }
  if (thresholds.requireAdjudicationForConflicts && missingAdjudication > 0) {
    addFailure(failures, "missingAdjudication", missingAdjudication, 0);
  }
  return {
    schemaVersion: 1,
    passed: failures.length === 0,
    metrics,
    failures,
  };
}

function parseJsonLines(contents, label) {
  return contents.split(/\r?\n/u).filter((line) => line.trim()).map((line, index) => {
    try {
      return JSON.parse(line);
    } catch (error) {
      throw new Error(`${label}:${index + 1} is not valid JSON: ${error.message}`);
    }
  });
}

function argument(arguments_, name, required = true) {
  const index = arguments_.indexOf(name);
  const value = index < 0 ? undefined : arguments_[index + 1];
  if (required && (!value || value.startsWith("--"))) throw new Error(`missing ${name}`);
  return value && !value.startsWith("--") ? value : undefined;
}

async function main(arguments_) {
  const reviewsPath = argument(arguments_, "--reviews");
  const adjudicationsPath = argument(arguments_, "--adjudications", false);
  const gatePath = argument(arguments_, "--gate");
  const outputPath = argument(arguments_, "--out", false);
  const [reviewsText, adjudicationsText, gateText] = await Promise.all([
    readFile(resolve(reviewsPath), "utf8"),
    adjudicationsPath ? readFile(resolve(adjudicationsPath), "utf8") : Promise.resolve(""),
    readFile(resolve(gatePath), "utf8"),
  ]);
  const result = evaluateReviewQuality(
    parseJsonLines(reviewsText, reviewsPath),
    adjudicationsText ? parseJsonLines(adjudicationsText, adjudicationsPath) : [],
    JSON.parse(gateText),
  );
  const serialized = `${JSON.stringify(result, null, 2)}\n`;
  if (outputPath) await writeFile(resolve(outputPath), serialized, "utf8");
  process.stdout.write(serialized);
  if (!result.passed) process.exitCode = 1;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(`review quality: ${error.message}`);
    process.exitCode = 2;
  });
}

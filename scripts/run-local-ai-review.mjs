#!/usr/bin/env node
import { createHash } from "node:crypto";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const RUBRIC = [
  "You are a Korean proofreading evaluator.",
  "Review only the supplied sentence. Do not rewrite its meaning or style.",
  "Return JSON only: {status:'normal'|'error'|'ambiguous', diagnostics:[{start,end,suggestion,errorFamily}] }.",
  "Ranges are UTF-8 byte offsets in the supplied sentence; end is exclusive.",
  "Use normal only when no spelling, spacing, grammar, or punctuation error is present.",
  "Use ambiguous when a correction depends on missing context. Never invent a correction.",
  "For each objective error, give the smallest changed range and a replacement suggestion.",
].join("\n");
const HASH = /^[0-9a-f]{64}$/u;

function sha256(value) {
  return createHash("sha256").update(String(value)).digest("hex");
}

function parseJsonLines(text, label) {
  return String(text).split(/\r?\n/u).filter((line) => line.trim()).map((line, index) => {
    try {
      return JSON.parse(line);
    } catch (error) {
      throw new Error(`${label}:${index + 1} is not valid JSON: ${error.message}`);
    }
  });
}

function utf8Boundaries(text) {
  const boundaries = new Set([0]);
  let offset = 0;
  for (const character of text) {
    offset += Buffer.byteLength(character, "utf8");
    boundaries.add(offset);
  }
  return boundaries;
}

function extractJson(value) {
  const text = String(value).replace(/```(?:json)?/giu, "").replace(/```/gu, "").trim();
  const start = text.indexOf("{");
  const end = text.lastIndexOf("}");
  if (start < 0 || end <= start) throw new Error("model response did not contain a JSON object");
  return JSON.parse(text.slice(start, end + 1));
}

function normalizeReview(raw, text) {
  const status = ["normal", "error", "ambiguous"].includes(raw?.status) ? raw.status : "ambiguous";
  const boundaries = utf8Boundaries(text);
  const diagnostics = [];
  if (status !== "normal" && Array.isArray(raw?.diagnostics)) {
    for (const item of raw.diagnostics) {
      const start = Number(item?.start);
      const end = Number(item?.end);
      const suggestion = typeof item?.suggestion === "string" ? item.suggestion : null;
      if (!Number.isInteger(start) || !Number.isInteger(end) || start >= end || !boundaries.has(start) || !boundaries.has(end) || suggestion === null) continue;
      diagnostics.push({
        ruleId: "ai.review",
        range: { start, end },
        suggestions: [suggestion],
        ...(typeof item.errorFamily === "string" && item.errorFamily.trim() ? { errorFamily: item.errorFamily.trim() } : {}),
      });
    }
  }
  if (status === "error" && diagnostics.length === 0) return { status: "ambiguous", diagnostics: [] };
  if (status === "normal") return { status, diagnostics: [] };
  return { status, diagnostics };
}

function packet({ caseId, reviewerId, modelSnapshot, text, review, rawResponse, sessionLabel }) {
  return {
    caseId,
    reviewerId,
    reviewerType: "ai",
    modelSnapshot,
    rubricSha256: sha256(RUBRIC),
    sessionSha256: sha256(sessionLabel),
    outputSha256: sha256(rawResponse),
    status: review.status,
    diagnostics: review.diagnostics,
  };
}

async function ollamaChat(baseUrl, model, messages, { temperature = 0.1 } = {}) {
  const response = await fetch(`${baseUrl.replace(/\/$/u, "")}/api/chat`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      model,
      stream: false,
      format: "json",
      options: { temperature, num_ctx: 512, num_predict: 64 },
      messages,
    }),
    signal: AbortSignal.timeout(120000),
  });
  if (!response.ok) throw new Error(`Ollama ${model} returned HTTP ${response.status}`);
  const body = await response.json();
  return { content: body.message?.content ?? "", model: body.model ?? model };
}

function reviewMessages(text) {
  return [
    { role: "system", content: RUBRIC },
    { role: "user", content: JSON.stringify({ sentence: text }) },
  ];
}

function batchReviewMessages(entries) {
  return [
    { role: "system", content: `${RUBRIC}\nYou are reviewing a batch. Return JSON only: {reviews:[{caseId,status,diagnostics}]}. Keep every caseId exactly as supplied and return one review per case.` },
    { role: "user", content: JSON.stringify({ sentences: entries.map((entry) => ({ caseId: entry.id, sentence: entry.text })) }) },
  ];
}

function adjudicationMessages(text, reviews) {
  return [
    { role: "system", content: `${RUBRIC}\nYou are the separate adjudicator. Compare the blinded candidate reviews. Select the most defensible review; if context is insufficient, return ambiguous.` },
    { role: "user", content: JSON.stringify({ sentence: text, candidateReviews: reviews.map(({ reviewerId, status, diagnostics }) => ({ reviewerId, status, diagnostics })) }) },
  ];
}

function batchAdjudicationMessages(entries, byCase) {
  return [
    { role: "system", content: `${RUBRIC}\nYou are the separate adjudicator for a batch. Return JSON only: {reviews:[{caseId,status,diagnostics}]}. Compare blinded candidate reviews and keep every caseId exactly as supplied.` },
    { role: "user", content: JSON.stringify({ cases: entries.map((entry) => ({
      caseId: entry.id,
      sentence: entry.text,
      candidateReviews: (byCase.get(entry.id) ?? []).map(({ reviewerId, status, diagnostics }) => ({ reviewerId, status, diagnostics })),
    })) }) },
  ];
}

export function chunkItems(items, size) {
  const chunks = [];
  for (let index = 0; index < items.length; index += size) chunks.push(items.slice(index, index + size));
  return chunks;
}

function batchReviewMap(rawResponse) {
  const value = extractJson(rawResponse);
  const reviews = Array.isArray(value?.reviews) ? value.reviews : [];
  return new Map(reviews.map((review) => [String(review?.caseId ?? review?.id ?? ""), review]));
}

async function mapLimit(items, limit, worker) {
  const output = new Array(items.length);
  let cursor = 0;
  async function run() {
    while (true) {
      const index = cursor;
      cursor += 1;
      if (index >= items.length) return;
      output[index] = await worker(items[index], index);
    }
  }
  await Promise.all(Array.from({ length: Math.min(limit, items.length) }, run));
  return output;
}

function parseArgs(argv) {
  const args = {};
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (!token.startsWith("--")) continue;
    args[token.slice(2)] = argv[index + 1] && !argv[index + 1].startsWith("--") ? argv[++index] : true;
  }
  return args;
}

async function main(argv) {
  const args = parseArgs(argv);
  if (!args.input || !args["reviews-out"] || !args["adjudications-out"]) {
    throw new Error("usage: node scripts/run-local-ai-review.mjs --input PATH --reviews-out PATH --adjudications-out PATH --models MODEL,MODEL,MODEL --adjudicator-model MODEL [--limit N]");
  }
  const models = String(args.models ?? "qwen2.5:3b,gemma3:1b,llama3.2:1b").split(",").map((model) => model.trim()).filter(Boolean);
  if (models.length < 3) throw new Error("at least three blind reviewer models are required");
  const adjudicatorModel = String(args["adjudicator-model"] ?? "qwen2.5:1.5b");
  const limit = Number(args.limit ?? 300);
  const concurrency = Number(args.concurrency ?? 4);
  const batchSize = Number(args["batch-size"] ?? 8);
  if (!Number.isInteger(limit) || limit <= 0) throw new Error("--limit must be a positive integer");
  if (!Number.isInteger(concurrency) || concurrency <= 0) throw new Error("--concurrency must be a positive integer");
  if (!Number.isInteger(batchSize) || batchSize <= 0 || batchSize > 32) throw new Error("--batch-size must be an integer from 1 to 32");
  const base = parseJsonLines(await readFile(resolve(args.input), "utf8"), args.input).slice(0, limit);
  if (base.length < 200) throw new Error("AI review requires at least 200 cases");
  const baseUrl = String(args["ollama-url"] ?? "http://localhost:11434");
  const reviews = [];
  const byCase = new Map();
  for (const model of models) {
    const batches = chunkItems(base, batchSize);
    const batchResults = await mapLimit(batches, concurrency, async (batch) => {
      let result;
      let rawResponse = "";
      try {
        result = await ollamaChat(baseUrl, model, batchReviewMessages(batch));
        rawResponse = result.content;
      } catch (error) {
        result = { model: model, content: `AI reviewer unavailable: ${error.message}` };
        rawResponse = result.content;
      }
      let reviewsByCase = new Map();
      try { reviewsByCase = batchReviewMap(rawResponse); } catch { /* every missing row becomes ambiguous below */ }
      return { batch, result, rawResponse, reviewsByCase };
    });
    const modelPackets = batchResults.flatMap(({ batch, result, rawResponse, reviewsByCase }) => batch.map((entry) => {
      const review = normalizeReview(reviewsByCase.get(entry.id), entry.text);
      return packet({
        caseId: entry.id,
        reviewerId: `ollama:${model}`,
        modelSnapshot: `ollama:${result.model}`,
        text: entry.text,
        review,
        rawResponse,
        sessionLabel: `geullint-local-ai-review-v1:${model}:${entry.id}`,
      });
    }));
    reviews.push(...modelPackets);
    for (const item of modelPackets) {
      const list = byCase.get(item.caseId) ?? [];
      list.push(item);
      byCase.set(item.caseId, list);
    }
  }
  const adjudications = [];
  const conflictEntries = base.filter((entry) => {
    const candidates = byCase.get(entry.id) ?? [];
    return new Set(candidates.map((item) => JSON.stringify({ status: item.status, diagnostics: item.diagnostics }))).size > 1;
  });
  const adjudicationBatches = await mapLimit(chunkItems(conflictEntries, batchSize), concurrency, async (batch) => {
    let result;
    let rawResponse = "";
    try {
      result = await ollamaChat(baseUrl, adjudicatorModel, batchAdjudicationMessages(batch, byCase), { temperature: 0 });
      rawResponse = result.content;
    } catch (error) {
      result = { model: adjudicatorModel, content: `AI adjudicator unavailable: ${error.message}` };
      rawResponse = result.content;
    }
    let reviewsByCase = new Map();
    try { reviewsByCase = batchReviewMap(rawResponse); } catch { /* every missing row becomes ambiguous below */ }
    return { batch, result, rawResponse, reviewsByCase };
  });
  const adjudicationPackets = adjudicationBatches.flatMap(({ batch, result, rawResponse, reviewsByCase }) => batch.map((entry) => packet({
    caseId: entry.id,
    reviewerId: `ollama:adjudicator:${adjudicatorModel}`,
    modelSnapshot: `ollama:${result.model}`,
    text: entry.text,
    review: normalizeReview(reviewsByCase.get(entry.id), entry.text),
    rawResponse,
    sessionLabel: `geullint-local-ai-adjudication-v1:${adjudicatorModel}:${entry.id}`,
  })));
  adjudications.push(...adjudicationPackets);
  await mkdir(dirname(resolve(args["reviews-out"])), { recursive: true });
  await writeFile(resolve(args["reviews-out"]), `${reviews.map(JSON.stringify).join("\n")}\n`, "utf8");
  await writeFile(resolve(args["adjudications-out"]), adjudications.length ? `${adjudications.map(JSON.stringify).join("\n")}\n` : "", "utf8");
  process.stdout.write(`${JSON.stringify({ cases: base.length, reviewers: models, adjudicatorModel, reviewPackets: reviews.length, adjudications: adjudications.length }, null, 2)}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(`local AI review: ${error.message}`);
    process.exitCode = 2;
  });
}

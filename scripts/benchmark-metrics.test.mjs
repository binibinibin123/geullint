import assert from "node:assert/strict";
import test from "node:test";
import { summarizeDurations } from "./benchmark-metrics.mjs";

test("summarizes sorted and unsorted samples with interpolated percentiles", () => {
  const summary = summarizeDurations([10, 2, 8, 4, 6], 2 * 1_024 * 1_024);

  assert.deepEqual(summary, {
    samples: 5,
    minMs: 2,
    meanMs: 6,
    p50Ms: 6,
    p95Ms: 9.6,
    throughputMiBPerSecond: 333.333,
  });
});

test("rejects empty, non-positive, and non-finite measurements", () => {
  for (const samples of [[], [0], [-1], [Number.NaN], [Number.POSITIVE_INFINITY]]) {
    assert.throws(() => summarizeDurations(samples, 1_024), /positive finite/u);
  }
  assert.throws(() => summarizeDurations([1], 0), /positive integer/u);
});

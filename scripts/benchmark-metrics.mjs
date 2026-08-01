function round(value) {
  return Number(value.toFixed(3));
}

function percentile(sorted, fraction) {
  const position = (sorted.length - 1) * fraction;
  const lower = Math.floor(position);
  const upper = Math.ceil(position);
  if (lower === upper) {
    return sorted[lower];
  }
  const weight = position - lower;
  return sorted[lower] * (1 - weight) + sorted[upper] * weight;
}

export function summarizeDurations(samples, byteLength) {
  if (!Array.isArray(samples)
    || samples.length === 0
    || samples.some((sample) => !Number.isFinite(sample) || sample <= 0)) {
    throw new TypeError("duration samples must contain positive finite numbers");
  }
  if (!Number.isSafeInteger(byteLength) || byteLength <= 0) {
    throw new TypeError("byteLength must be a positive integer");
  }

  const sorted = [...samples].sort((left, right) => left - right);
  const mean = sorted.reduce((total, sample) => total + sample, 0) / sorted.length;
  const median = percentile(sorted, 0.5);
  const mebibytes = byteLength / (1_024 * 1_024);

  return {
    samples: sorted.length,
    minMs: round(sorted[0]),
    meanMs: round(mean),
    p50Ms: round(median),
    p95Ms: round(percentile(sorted, 0.95)),
    throughputMiBPerSecond: round(mebibytes / (median / 1_000)),
  };
}

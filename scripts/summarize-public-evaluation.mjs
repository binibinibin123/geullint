import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

function readText(filePath) {
  const buffer = fs.readFileSync(filePath);
  if (buffer.length >= 2 && buffer[0] === 0xff && buffer[1] === 0xfe) {
    return buffer.toString('utf16le').replace(/^\ufeff/, '');
  }
  return buffer.toString('utf8').replace(/^\ufeff/, '');
}

function readJsonLines(filePath) {
  return readText(filePath)
    .split(String.fromCharCode(10))
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => JSON.parse(line));
}

function countBy(items, selector) {
  return Object.fromEntries(
    [...items.reduce((counts, item) => {
      const value = selector(item) ?? 'unattributed';
      counts.set(value, (counts.get(value) ?? 0) + 1);
      return counts;
    }, new Map())].sort(([a], [b]) => a.localeCompare(b)),
  );
}

function exactFixedTextStats(cases, mismatchIds) {
  const eligible = cases.filter((item) => item.expectedFixedText);
  const mismatches = eligible.filter((item) => mismatchIds.has(item.id));
  const matches = eligible.length - mismatches.length;
  return {
    cases: eligible.length,
    exactMatches: matches,
    exactAccuracy: eligible.length === 0 ? null : matches / eligible.length,
    mismatches: mismatches.length,
  };
}

function summarizeSlices(cases, failures, selector) {
  const failureById = new Map(failures.map((failure) => [failure.id, failure]));
  const slices = new Map();
  for (const item of cases) {
    const values = selector(item);
    for (const value of values) {
      const slice = slices.get(value) ?? {
        cases: 0,
        normalCases: 0,
        correctionCases: 0,
        falsePositiveCases: 0,
        falseNegativeCases: 0,
        fixedTextMismatches: 0,
      };
      slice.cases += 1;
      const normal = item.caseType === 'normal';
      if (normal) slice.normalCases += 1;
      else slice.correctionCases += 1;
      const failure = failureById.get(item.id);
      if (failure?.falsePositiveRuleIds?.length) slice.falsePositiveCases += 1;
      if (failure?.falseNegativeRuleIds?.length) slice.falseNegativeCases += 1;
      if (failure?.fixedTextMismatch) slice.fixedTextMismatches += 1;
      slices.set(value, slice);
    }
  }
  return Object.fromEntries([...slices.entries()].sort(([left], [right]) => left.localeCompare(right)).map(([key, slice]) => [
    key,
    {
      ...slice,
      safeNormalAccuracy: slice.normalCases === 0 ? null : (slice.normalCases - slice.falsePositiveCases) / slice.normalCases,
      diagnosticCorrectionRecall: slice.correctionCases === 0 ? null : (slice.correctionCases - slice.falseNegativeCases) / slice.correctionCases,
      exactFixedTextAccuracy: slice.correctionCases === 0 ? null : (slice.correctionCases - slice.fixedTextMismatches) / slice.correctionCases,
    },
  ]));
}

export function summarizePublicEvaluation(cases, nativeReport) {
  const failures = Array.isArray(nativeReport.caseFailures) ? nativeReport.caseFailures : [];
  const fixedTextMismatchIds = new Set(
    failures
      .filter((failure) => failure.kind === 'fixedTextMismatch' || failure.fixedTextMismatch)
      .map((failure) => failure.id),
  );
  const sourceRevisions = cases.filter((item) => item.annotationOrigin === 'source_revision');
  const synthetic = cases.filter((item) => item.textOrigin === 'synthetic');
  const independentHuman = cases.filter((item) => item.annotationOrigin === 'human_independent');
  const sourceRevisionStats = exactFixedTextStats(sourceRevisions, fixedTextMismatchIds);
  const syntheticStats = exactFixedTextStats(synthetic, fixedTextMismatchIds);
  const independentHumanStats = exactFixedTextStats(independentHuman, fixedTextMismatchIds);

  return {
    schemaVersion: 1,
    cases: nativeReport.cases ?? cases.length,
    normalCases: cases.filter((item) => item.caseType === 'normal').length,
    sourceRevisionCases: sourceRevisions.length,
    syntheticCases: synthetic.length,
    syntheticExactFixedTextMatches: syntheticStats.exactMatches,
    syntheticExactFixedTextAccuracy: syntheticStats.exactAccuracy,
    independentHumanCases: independentHuman.length,
    sourceRevisionExactFixedTextMatches: sourceRevisionStats.exactMatches,
    sourceRevisionExactFixedTextAccuracy: sourceRevisionStats.exactAccuracy,
    independentHumanExactFixedTextMatches: independentHumanStats.exactMatches,
    independentHumanExactFixedTextAccuracy: independentHumanStats.exactAccuracy,
    splits: countBy(cases, (item) => item.split),
    genres: countBy(cases, (item) => item.genre),
    authors: new Set(cases.map((item) => item.authorId).filter(Boolean)).size,
    documents: new Set(cases.map((item) => item.documentId).filter(Boolean)).size,
    native: {
      specificity: nativeReport.specificity ?? null,
      falsePositiveCases: nativeReport.falsePositiveCases ?? null,
      precision: nativeReport.precision ?? null,
      recall: nativeReport.recall ?? null,
      macroPrecision: nativeReport.macroPrecision ?? null,
      macroRecall: nativeReport.macroRecall ?? null,
      top1CorrectionAccuracy: nativeReport.top1CorrectionAccuracy ?? null,
      top5CorrectionAccuracy: nativeReport.top5CorrectionAccuracy ?? null,
    },
    byGenre: summarizeSlices(cases, failures, (item) => [item.genre ?? 'unattributed']),
    byErrorFamily: summarizeSlices(cases, failures, (item) => Array.isArray(item.errorFamilies) && item.errorFamilies.length ? item.errorFamilies : ['none']),
  };
}

function parseArgs(argv) {
  const args = {};
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (!token.startsWith('--')) continue;
    args[token.slice(2)] = argv[index + 1] && !argv[index + 1].startsWith('--')
      ? argv[++index]
      : true;
  }
  return args;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = parseArgs(process.argv.slice(2));
  if (!args.corpus || !args['native-report'] || !args.out) {
    throw new Error('Usage: node scripts/summarize-public-evaluation.mjs --corpus PATH --native-report PATH --out PATH');
  }
  const summary = summarizePublicEvaluation(
    readJsonLines(path.resolve(args.corpus)),
    JSON.parse(readText(path.resolve(args['native-report']))),
  );
  fs.mkdirSync(path.dirname(path.resolve(args.out)), { recursive: true });
  fs.writeFileSync(path.resolve(args.out), `${JSON.stringify(summary, null, 2)}\n`, 'utf8');
  process.stdout.write(`${JSON.stringify(summary, null, 2)}\n`);
}

import test from 'node:test';
import assert from 'node:assert/strict';

import { summarizePublicEvaluation } from './summarize-public-evaluation.mjs';

test('summarizes exact source-revision correction accuracy separately from rule metrics', () => {
  const cases = [
    {
      id: 'normal-1',
      caseType: 'normal',
      split: 'train',
      genre: 'general',
      text: '오늘은 맑다.',
    },
    {
      id: 'source-1',
      caseType: 'correction',
      split: 'H1',
      annotationOrigin: 'source_revision',
      expectedFixedText: '오늘은 맑았다.',
      genre: 'education',
      text: '오늘은 맑다',
    },
    {
      id: 'source-2',
      caseType: 'correction',
      split: 'H2',
      annotationOrigin: 'source_revision',
      expectedFixedText: '비가 온다.',
      genre: 'dialogue',
      text: '비가온다.',
    },
    {
      id: 'human-1',
      caseType: 'correction',
      split: 'H1',
      annotationOrigin: 'human_independent',
      expectedFixedText: '사람이 검토했다.',
      genre: 'general',
      text: '사람이 검토햇다.',
    },
  ];
  const report = {
    cases: 4,
    specificity: 0.9,
    falsePositiveCases: 1,
    fixedTextCases: 3,
    exactFixedTextHits: 1,
    exactFixedTextAccuracy: 1 / 3,
    correctionDetectionHits: 2,
    correctionDetectionRecall: 2 / 3,
    caseFailures: [
      { id: 'source-2', kind: 'fixedTextMismatch', correctionDetectionMiss: true },
      { id: 'human-1', kind: 'fixedTextMismatch' },
    ],
  };

  const summary = summarizePublicEvaluation(cases, report);

  assert.equal(summary.cases, 4);
  assert.equal(summary.normalCases, 1);
  assert.equal(summary.sourceRevisionCases, 2);
  assert.equal(summary.sourceRevisionExactFixedTextMatches, 1);
  assert.equal(summary.sourceRevisionExactFixedTextAccuracy, 0.5);
  assert.equal(summary.independentHumanCases, 1);
  assert.equal(summary.independentHumanExactFixedTextMatches, 0);
  assert.deepEqual(summary.splits, { H1: 2, H2: 1, train: 1 });
  assert.equal(summary.native.specificity, 0.9);
  assert.equal(summary.native.fixedTextCases, 3);
  assert.equal(summary.native.exactFixedTextAccuracy, 1 / 3);
  assert.equal(summary.native.correctionDetectionRecall, 2 / 3);
});

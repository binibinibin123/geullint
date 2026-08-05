# AI-adjudicated evaluation pipeline report

Status: **development infrastructure complete; commercial claim NO-GO**

This report describes the evaluation machinery added in the `ai-adjudicated-eval`
branch. It does not turn the checked-in 144-case regression corpus into an
independent human benchmark.

## Implemented checks

- JSONL v2 separates `textOrigin` and `annotationOrigin` and preserves
  reviewer, model, rubric, session, and output hashes.
- A/B/C blind packets merge only on unanimous normalized status/diagnostics.
  Conflicts require a separate adjudicator; unresolved cases become
  `ambiguous` with no forced diagnostics.
- Document, author, source, exact text, NFKC/whitespace, and decomposed-Hangul
  5-gram leakage checks run before metrics. H1 and H2 carry matching
  `holdoutId` values.
- Rust reports include specificity, top-1/top-5 correction accuracy, Wilson
  lower bounds, author/holdout counts, and genre/error-family case maps.
- Review-quality reports track agreement, adjudication rate, audit disagreement,
  reviewer count, non-AI packets, duplicates, and missing hashes.
- Native/WASM parity remains a release check; the commercial wrapper accepts
  parity, leakage, and review-quality results as auxiliary gates.

## Current gate result

The checked-in `corpus/safety-regressions-v1.jsonl` is intentionally a project
regression corpus. It is useful for deterministic safety checks but has no
independent document/author holdout and no independent human annotation layer.
Running:

```powershell
node scripts/evaluate-commercial-gate.mjs `
  --cli target/debug/geullint.exe `
  --corpus corpus/safety-regressions-v1.jsonl `
  --gate corpus/gates/commercial-near-v1.json
```

must exit `1` and report `qualityGate.passed: false`. The failure is expected:
the required natural-case, human-edit, author, H1, H2, and independent-human
minimums are not present. No release note or README should call this result
commercial-equivalent, Harper-equivalent, or Naver-equivalent.

## Promotion rule

Only an authorized source with a verified manifest hash may populate H1/H2.
AI-adjudicated rows can measure model agreement and candidate correction quality,
but a release remains NO-GO until each holdout has the required independent
human evidence and passes the Safe/specificity/top-k and Native/WASM gates.

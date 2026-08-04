# v0.3.0-alpha.2 quality report

This report describes what this release actually verifies. It is not a claim of parity with a commercial checker or Harper.

## Catalogue

- catalogue: 116 rules
- New high-confidence lexical coverage: `데이타 → 데이터`, `설레임 → 설렘`, `내노라하는 → 내로라하는`
- Contextual review-only coverage: `바램 → 바람`
- Dependent-noun spacing coverage includes distinct `수`, `것`, and `적` forms.
- Particle allomorph suggestions use the complete preceding word range and remain review-only.

## Regression fixtures

The repository-owned fixtures remain green:

| fixture | error sentences | normal sentences | purpose |
| --- | ---: | ---: | --- |
| safety-regressions-v1 | 72 | 72 | safe-fix and source-boundary gate |
| curated-alpha-v1 | 84 | 42 | per-rule catalogue contract |
| KoLLA v2 review slice | — | 249 | no-op specificity smoke check |

These fixtures are authored or curated for regression protection. They are not an independent estimate of general Korean accuracy.

## Limits

- OOV candidate generation remains out of scope; unknown names, slang, and new compounds need a user dictionary or a review suggestion.
- The engine is conservative and rule-based. It is not a Harper equivalent and is not advertised as one.
- Review-only suggestions are never applied by the safe-fix path unless the user explicitly opts in.

## Reproduce

```bash
cargo test --workspace --all-features
node --test scripts/*.test.mjs
node scripts/validate-safety-corpus.mjs \
  --corpus corpus/safety-regressions-v1.jsonl \
  --policy corpus/safety-regressions-v1.policy.json \
  --cli target/debug/geullint
```

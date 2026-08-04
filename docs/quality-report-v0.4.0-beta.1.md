# GeulLint v0.4.0-beta.1 quality report

This beta report records reproducible checks and known limits. It does not claim parity with a commercial checker or Harper.

## Catalogue

- catalogue: 116 rules
- The standard engine bundles the reviewed lexical core and keeps context-ranked candidates review-only.
- Safe fixes remain conservative; user dictionaries and explicit review mode cover project-specific names and uncertain suggestions.

## Regression fixtures

The repository-owned fixtures remain green:

| fixture | error sentences | normal sentences | purpose |
| --- | ---: | ---: | --- |
| safety-regressions-v1 | 72 | 72 | safe-fix and source-boundary gate |
| curated-alpha-v1 | 84 | 42 | per-rule catalogue contract |
| KoLLA v2 review slice | 249 | 0 | training-only review slice |

These fixtures protect regressions. They are not an independent estimate of general Korean accuracy.

## Verified release paths

- Rust workspace tests and all-features checks pass.
- Native, WASM, browser, CLI, LSP, and VS Code paths use the same diagnostic contract.
- The Chromium desktop/mobile offline E2E passes with zero external requests.
- Release assets are checked for reproducible hashes across Windows and Linux hosts.
- Browser dictionary and draft persistence survives an immediate reload while IndexedDB is opening.

## Accuracy boundary

- OOV candidate generation remains out of scope; unknown names, slang, and new compounds need a user dictionary or a review suggestion.
- The engine is conservative and rule-based. It is not a Harper equivalent and is not advertised as one.
- `commercial-near-v1` remains NO-GO until an authorized independent human holdout is supplied. The required 20,000 natural cases, 5,000 human edits, 10,000 normal cases, eight genres, and release holdout are not fabricated here.

## Reproduce

```bash
cargo test --workspace --all-features
node --test scripts/*.test.mjs
node scripts/validate-safety-corpus.mjs \
  --corpus corpus/safety-regressions-v1.jsonl \
  --policy corpus/safety-regressions-v1.policy.json \
  --cli target/debug/geullint
```

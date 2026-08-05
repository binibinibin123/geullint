# AI-Adjudicated Evaluation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an auditable blind AI review and holdout evaluation pipeline without mislabeling AI review as human review.

**Architecture:** Keep the current Rust evaluator and KoLLA human-source curation path compatible, then add a versioned JSONL v2 provenance layer, a separate AI review merger, complete document/author leakage checks, and two locked holdout gates. The engine remains deterministic and all public claims are derived from verified manifests.

**Tech Stack:** Rust 2024, Node.js 22 ESM scripts, JSON Schema-style validation, SHA-256 provenance, existing CLI corpus evaluator, Node test runner.

---

### Task 1: Lock the provenance contract with failing tests

**Files:**
- Create: `scripts/ai-adjudication.test.mjs`
- Create: `corpus/schema/case-v2.schema.json`
- Modify: `scripts/data-pipeline.test.mjs`

- [ ] Add tests rejecting an AI row labeled `annotationOrigin: "human_independent"` or `origin: "independent_human"` without human evidence.
- [ ] Add tests accepting normal rows with `expectedDiagnostics: []`, ambiguous rows with `annotationStatus: "ambiguous"`, and multiple allowed corrections.
- [ ] Add tests requiring `textOrigin`, `annotationOrigin`, `annotationStatus`, `holdoutId`, reviewer type, and all provenance hashes for non-project rows.
- [ ] Run `node --test scripts/ai-adjudication.test.mjs scripts/data-pipeline.test.mjs` and confirm the new tests fail for the missing schema/validator.
- [ ] Commit the red tests as `test: define AI adjudication provenance contract`.

### Task 2: Implement JSONL v2 validation and AI review merging

**Files:**
- Create: `scripts/ai-adjudication.mjs`
- Create: `scripts/merge-ai-reviews.mjs`
- Modify: `scripts/curate-kolla-v2-gold.mjs`
- Modify: `scripts/data-pipeline.test.mjs`

- [ ] Implement strict enums and required fields from the schema, keeping KoLLA’s existing human-review validator separate.
- [ ] Validate A/B/C reviews by exact normal/error status, UTF-8 byte ranges, normalized correction sets, and reviewer type/model/session/output hashes.
- [ ] Promote unanimous cases, route disagreements and high-risk flags to a separate adjudicator record, and route unresolved cases to `ambiguous` without expected diagnostics.
- [ ] Preserve `textOrigin`, `genre`, `split`, `documentId`, `authorId`, `holdoutId`, and review provenance in the promoted JSONL.
- [ ] Reject any attempt to count `ai_blind_panel` as `independent_human`.
- [ ] Run the focused tests and confirm green.
- [ ] Commit as `feat: add auditable blind AI review merger`.

### Task 3: Make splitting and leakage checks complete

**Files:**
- Create: `scripts/split-corpus-by-document.mjs`
- Create: `scripts/import-evaluation-sources.mjs`
- Modify: `scripts/check-corpus-leakage.mjs`
- Modify: `scripts/corpus-leakage.test.mjs`

- [ ] Split by source document, author, and revision lineage before sentence sampling; require H1/H2 identifiers.
- [ ] Add NFKC/whitespace/punctuation normalization, real Hangul jamo decomposition, complete candidate comparison without a fixed 512-item cap, and duplicate document/author/source checks.
- [ ] Reject KoLLA training document IDs from either holdout and reject project/synthetic rows from commercial holdouts.
- [ ] Verify locked manifest hashes before any evaluator run.
- [ ] Run leakage and source-manifest tests and confirm green.
- [ ] Commit as `test: close corpus leakage and holdout boundaries`.

### Task 4: Extend evaluator metrics and gates

**Files:**
- Modify: `crates/geullint-cli/src/evaluation_v2.rs`
- Modify: `crates/geullint-cli/src/main.rs`
- Modify: `corpus/gates/commercial-near-v1.json`
- Create: `corpus/gates/model-adjudicated-v1.json`
- Modify: `scripts/evaluate-commercial-gate.mjs`
- Create: `scripts/evaluate-review-quality.mjs`
- Create: `scripts/quality-gates.test.mjs`

- [ ] Add annotation provenance, holdout ID, minimum authors, minimum holdout cases, specificity, top-1/top-5 correction accuracy, family/genre slices, Safe precision Wilson lower bound, and parity results to the report.
- [ ] Count only `human_independent`/`source_revision` rows as human-origin edits; do not count `needs_adjudication` rows.
- [ ] Require both H1 and H2 to pass independently and require a fresh holdout ID for a second release.
- [ ] Replace the misleading per-rule expected-count-only Safe threshold with an explicit prediction denominator and Wilson lower-bound requirement.
- [ ] Make the wrapper run manifest verification, leakage verification, slice metrics, and Native/WASM parity before producing a pass.
- [ ] Add review-quality gates for agreement, adjudication rate, audit disagreement, and missing provenance hashes.
- [ ] Run Rust CLI tests and focused gate tests; confirm the existing 144-case safety corpus still reports NO-GO for commercial claims.
- [ ] Commit as `feat: enforce dual holdout quality gates`.

### Task 5: Add review packet/report documentation

**Files:**
- Create: `docs/ai-adjudication.md`
- Modify: `docs/accuracy-methodology.md`
- Modify: `docs/corpus-sources.md`
- Modify: `docs/data-card.md`
- Modify: `docs/release-go-no-go.md`
- Modify: `README.md`

- [ ] Document the A/B/C/D/E blind protocol, batch size, hashes, ambiguity policy, and operator boundaries.
- [ ] State clearly that AI-adjudicated data is not human-reviewed evidence and cannot by itself support a commercial-equivalence claim.
- [ ] Document the required H1/H2 counts, metrics, source licenses, and holdout storage boundary.
- [ ] Update the release checklist to display the exact gate name and NO-GO reason when independent human evidence is absent.
- [ ] Run README, corpus, and repository-health contract tests.
- [ ] Commit as `docs: publish AI adjudication and holdout methodology`.

### Task 6: Full verification and release boundary

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `.github/workflows/release.yml`
- Modify: `CHANGELOG.md`
- Create: `docs/quality-report-ai-adjudicated-v1.md`

- [ ] Add CI jobs for schema, leakage, review-quality, Rust evaluator, Node scripts, and existing offline browser contracts.
- [ ] Run `node --test scripts/*.test.mjs`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace --all-features --no-fail-fast`.
- [ ] Run the evaluator against the current checked-in corpus and record the expected commercial NO-GO without fabricating holdout counts.
- [ ] Verify working tree cleanliness, PR checks, and release workflow configuration.
- [ ] Publish only a development-quality report unless real H1/H2 and independent human evidence exist; do not bump a public release tag for an unverified claim.
- [ ] Commit as `test: verify AI-adjudicated evaluation boundary`.

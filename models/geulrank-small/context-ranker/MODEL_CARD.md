# GeulRank-small context ranker

This directory contains an experimental, dependency-free local context ranker.

- Format: `geulrank-context-linear-int8-v1`.
- Runtime artifact: `context-linear-int8.json` (native/WASM).
- Interchange artifact: `context-linear-int8.onnx` (`MatMulInteger`, INT8 weights).
- Features: deterministic hashed source/candidate character n-grams plus bounded edit features.
- Training source: KoLLA v2 annotation pairs only; rows are training material, not independently adjudicated release gold.
- Release holdout rows are rejected by the trainer and excluded from the checked-in artifact.

The model is opt-in through the `context` CLI engine, `StandardPipeline::bundled_with_context`,
and `lint_context_json`. All generated candidates remain `Review-only`; the default deterministic
pipeline and Safe-fix policy are unchanged until an independent human-adjudicated holdout passes.

Rebuild after acquiring the source data:

```bash
PYTHONPATH=training python -m geullint_training.kolla_pairs \
  --input data/raw/kolla-v2/KoLLA_multi-refs.m2 \
  --output data/derived/kolla-v2-pairs.jsonl
PYTHONPATH=training python -m geullint_training.train_context_ranker \
  --input data/derived/kolla-v2-pairs.jsonl \
  --out-dir models/geulrank-small/context-ranker
```

The manifest records SHA-256 hashes, feature dimensions, training row count, and the explicit
training-only provenance.

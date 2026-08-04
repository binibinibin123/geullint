# 모델·사전 배포

GeulLint의 실행 경로는 모델이나 사전을 동적으로 내려받지 않는다. 릴리스 아카이브, VSIX, PWA에 포함되는 파일은 release manifest와 SHA-256으로 고정한다.

현재 `models/geulrank-small`은 Rust/WASM이 직접 읽는 결정론적 INT8 baseline이다. `onnx: false`인 동안에는 ONNX 런타임을 의존성으로 끌어들이지 않는다. 실제 학습 모델을 추가할 때는 다음 항목을 한 커밋에서 갱신한다.

- 모델·tokenizer·calibration의 파일별 SHA-256과 크기
- 학습 데이터 manifest, 라이선스, 문서 ID 분할 결과
- Native/WASM 점수 parity와 독립 dev/holdout 지표
- 모델 카드의 알려진 오류, 편향, 복구 경로

## standard 실행 경로의 현재 상태

`geullint-core`의 `standard` feature를 켜면 `StandardPipeline::bundled`가 versioned lexicon과
portable ranker를 실제 후보 생성 단계에 연결한다. spelling/spacing 후보는 현재 모두 `Review`로만
노출되고 `fixedText`에는 적용되지 않는다. 독립 release holdout에서 안전도와 재현율을 측정하기
전까지 CLI·웹의 compact 기본 경로를 바꾸지 않는 것이 의도된 안전 경계다.

```bash
cargo test -p geullint-core --all-features --test standard_pipeline
```

이 baseline은 학습된 cross-encoder나 ONNX 모델을 의미하지 않는다. `models/geulrank-small/manifest.json`
의 `deterministic-baseline` 상태와 품질 보고서를 함께 확인해야 한다.

## Experimental context ranker

`models/geulrank-small/context-ranker/` contains a reproducible INT8
`MatMulInteger` ONNX artifact and matching dependency-free JSON weights. It is
trained only from KoLLA annotation pairs, excludes release holdout rows, and is
marked training-only in its manifest and model card. The default product path
remains the deterministic portable ranker; context-ranker results are not
promoted to Safe or used for a quality claim until independent adjudicated
holdouts pass.

The runtime surfaces are `geullint check --engine standard`,
`StandardPipeline::bundled`, `evaluate_standard`, and `lint_standard_json`.
The browser playground builds the standard feature and exposes `standard`,
`compact`, and the experimental `context` engine in its local selector. Standard
and context candidates remain Review-only; the compact engine remains available
for the smallest embedding and the VS Code Safe-fix path remains conservative.

`geullint check --engine context`, `StandardPipeline::bundled_with_context`,
`evaluate_context`, and `lint_context_json` are intended for experimentation and
ranking analysis. No learned candidate is promoted to Safe until an independent
adjudicated holdout passes.

브라우저는 최초 정적 자산 설치 뒤 Worker와 Service Worker 캐시만 사용한다. 캐시가 손상되거나 무결성 검증이 실패하면 검사를 시작하지 않고 사용자에게 재설치를 안내한다.

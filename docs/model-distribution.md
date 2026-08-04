# 모델·사전 배포

GeulLint의 실행 경로는 모델이나 사전을 동적으로 내려받지 않는다. 릴리스 아카이브, VSIX, PWA에 포함되는 파일은 release manifest와 SHA-256으로 고정한다.

현재 `models/geulrank-small`은 Rust/WASM이 직접 읽는 결정론적 INT8 baseline이다. `onnx: false`인 동안에는 ONNX 런타임을 의존성으로 끌어들이지 않는다. 실제 학습 모델을 추가할 때는 다음 항목을 한 커밋에서 갱신한다.

- 모델·tokenizer·calibration의 파일별 SHA-256과 크기
- 학습 데이터 manifest, 라이선스, 문서 ID 분할 결과
- Native/WASM 점수 parity와 독립 dev/holdout 지표
- 모델 카드의 알려진 오류, 편향, 복구 경로

브라우저는 최초 정적 자산 설치 뒤 Worker와 Service Worker 캐시만 사용한다. 캐시가 손상되거나 무결성 검증이 실패하면 검사를 시작하지 않고 사용자에게 재설치를 안내한다.

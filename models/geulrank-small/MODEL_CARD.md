# GeulRank-small

GeulRank-small은 후보를 고르는 로컬 랭킹 계층의 버전 계약이다. 현재 저장소에는 음절 편집 거리·발음 유사도·사전 빈도·기존 후보 점수를 사용하는 결정론적 INT8 기준선이 들어 있다.

현재 체크인된 `geulrank-small.int8.json`은 Rust와 WASM이 직접 읽을 수 있는 164바이트 portable baseline이다. 10~15M parameter cross-encoder나 ONNX 런타임 모델을 독립 holdout으로 검증하기 전까지는 이 파일을 학습 모델이라고 부르지 않으며, 베타 품질 수치의 근거로 사용하지 않는다.

## 안전한 사용 범위

- 모델은 문장을 외부로 보내지 않는다.
- 모델 점수만으로 자동 수정하지 않는다. `Safe/Review/Abstain` 정책과 충돌 해결기가 최종 동작을 결정한다.
- 학습 데이터는 문서 ID 기준으로 분리하고 `release_holdout`을 학습에서 제외한다.
- 사람의 수정본은 두 명 이상 독립 검토와 adjudication을 거치기 전에는 골드 정답이 아니다.

## 기준선과 교체 조건

`manifest.json`의 `format`과 feature 이름은 Native·WASM·웹에서 공통으로 유지한다. 학습 모델을 교체할 때는 모델 해시, 데이터 매니페스트 해시, calibration 결과, Native/Web parity 결과를 함께 갱신한다. 독립 holdout gate를 통과하지 못한 모델은 릴리스 자산으로 올리지 않는다.

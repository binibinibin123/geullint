# AI 다중 블라인드 한국어 평가 설계

## 목표

GeulLint의 실제 문장 평가를 확대하되 AI가 수행한 검수를 인간 검수로 표기하지 않는다. 개발용 평가와 상용 품질 주장을 서로 다른 게이트로 분리하고, 같은 문장·작성자·출처가 학습과 holdout 사이에 섞이지 않도록 한다.

## 핵심 원칙

- `textOrigin`과 `annotationOrigin`을 분리한다.
- AI 검수는 `ai_blind_panel`로만 기록한다. `independent_human`은 증빙된 외부 인간 주석에만 사용한다.
- 엔진 출력과 규칙 ID를 숨긴 A/B/C 검수 세션을 먼저 실행하고, 불일치는 D adjudicator가 판정한다.
- 정상·오류·모호를 모두 표현한다. 모호한 행을 정답으로 강제하지 않는다.
- holdout 원문은 개발자에게 노출하지 않고 H1/H2를 서로 다른 잠금 세트로 유지한다.
- 정확도 수치와 릴리스 주장은 provenance, 누수 검사, 검수 품질 지표가 모두 통과한 경우에만 생성한다.

## 데이터 흐름

```text
licensed sources
  -> document/author split
  -> immutable manifest + hashes
  -> blind review packets
  -> A/B/C reviews
  -> D adjudication + E audit
  -> reviewed JSONL v2
  -> leakage + provenance validation
  -> development gate or H1/H2 holdout gate
```

검수 패킷은 40~50개 문장 단위로 만들며 엔진 진단·규칙 ID·기존 검수 결과를 포함하지 않는다. 각 검수 결과에는 모델 snapshot, rubric hash, session hash, output hash를 남긴다.

## 스키마 경계

`textOrigin`은 `human_authored`, `revision`, `project`, `synthetic` 중 하나다. `annotationOrigin`은 `ai_blind_panel`, `human_independent`, `source_revision` 중 하나다. `annotationStatus`는 `unreviewed`, `reviewed`, `adjudicated`, `ambiguous` 중 하나다. `reviewerType`과 `adjudicatorType`은 `ai` 또는 `human`이어야 한다.

`holdoutId`는 `H1` 또는 `H2`이며, holdout 행은 문서·작성자·출처 단위로 개발 split과 겹치지 않는다. AI 검수 행에는 `independentHumanEvidence`를 기록하지 않는다.

## 게이트

`model-adjudicated-v1`은 개발용 게이트다. 20,000개 이상의 자연 문장, 5,000개 이상의 실제 인간 수정 원천, 10,000개 이상의 실제 정상 문장, 8개 장르, H1/H2 각각 10,000개를 요구한다. 검수 합의도는 Fleiss κ 또는 Krippendorff α 0.80 이상, span F1 0.90 이상, 교정안 일치율 0.85 이상, adjudication 비율 20% 이하, 재감사 불일치 2% 이하를 요구한다.

`commercial-near-v2`는 독립 인간 증빙이 없는 경우 항상 NO-GO다. 통과 가능한 릴리스는 micro precision 0.98 이상, recall 0.85 이상, specificity 0.995 이상, top-1 0.92 이상, top-5 0.98 이상, family별 recall 0.80 이상, Safe precision Wilson 하한 0.995 이상, Native/WASM parity 100%를 H1/H2에서 각각 만족해야 한다. Safe family는 예상 수 50개가 아니라 최소 예측 분모를 별도로 충족해야 한다.

## 구현 경계

기존 KoLLA 큐레이션 도구는 인간 원천 자료 전용으로 유지한다. 새 AI 패널 도구는 독립 파일로 만들고, 기존 release-quality curation 경로에 AI 결과가 섞이지 않게 한다. 원문 holdout은 저장소에 커밋하지 않으며 manifest와 집계 보고서만 공개한다.

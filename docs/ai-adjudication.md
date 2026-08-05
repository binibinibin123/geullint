# AI 다중 블라인드 검수와 독립 평가 경계

이 저장소의 AI 검수 파이프라인은 평가용 주석을 만들기 위한 도구입니다. AI가 만든 주석은 인간 독립 검수의 증거가 아니며, AI 검수만으로 상용 검사기와 동급이라는 주장을 하지 않습니다.

## 데이터와 주석의 분리

각 v2 행은 다음 두 출처를 분리해 기록합니다.

- `textOrigin`: 원문이 사람이 작성한 것인지(`human_authored`), revision인지, 프로젝트 fixture인지, synthetic인지
- `annotationOrigin`: 주석이 AI 블라인드 패널인지(`ai_blind_panel`), 독립 인간인지(`human_independent`), 원문 revision인지
- `annotationStatus`: `unreviewed`, `reviewed`, `adjudicated`, `ambiguous`
- `reviewProvenance`: reviewer/adjudicator 유형, 모델 snapshot, rubric·session·output SHA-256

`ai_blind_panel`은 `reviewerType: "ai"`, `adjudicatorType: "ai"`, 서로 다른 모델 snapshot 두 개 이상을 요구합니다. `humanEvidence`를 넣거나 `human_independent`로 바꾸면 validator가 거부합니다. 반대로 `human_independent`는 `reviewerType: "human"`과 별도의 `humanEvidence`가 있어야 합니다.

## 블라인드 검수 순서

1. 원문 문서·작성자 단위로 먼저 분할하고, 문장 ID를 생성합니다.
2. 같은 원문을 모델 A/B/C에 독립적으로 보내며 모델 출력에는 엔진의 정답을 섞지 않습니다.
3. 각 packet에 모델 snapshot, rubric, 세션, 출력의 SHA-256과 `normal`/`error`/`ambiguous` 상태를 기록합니다.
4. 모든 packet이 같으면 `reviewed`로 승격합니다.
5. 다르면 별도 adjudicator packet을 요구합니다. 합의되지 않으면 `ambiguous`로 남기고 `expectedDiagnostics`를 비웁니다.

예시 도구:

```powershell
node scripts/merge-ai-reviews.mjs `
  --cases source-cases.jsonl `
  --reviews blind-reviews.jsonl `
  --adjudications adjudications.jsonl `
  --out evaluated-v2.jsonl
node scripts/evaluate-review-quality.mjs `
  --reviews blind-reviews.jsonl `
  --adjudications adjudications.jsonl `
  --gate corpus/gates/model-adjudicated-v1.json
```

검수 품질 보고서는 agreement rate, adjudication rate, adjudication disagreement rate, 누락된 provenance hash, reviewer 수를 별도로 출력합니다. 한 명의 AI가 여러 번 답한 것은 독립 검수자 수로 세지 않습니다.

## H1/H2 holdout과 누수 방지

`node scripts/split-corpus-by-document.mjs`는 author/document 그룹을 먼저 해시해 `train`, `dev`, `H1`, `H2`에 배정합니다. H1/H2 행은 반드시 같은 `holdoutId`를 가져야 합니다. `scripts/check-corpus-leakage.mjs`는 다음을 모두 검사합니다.

- exact text, NFKC·공백 정규화 text, UTF-8 한글 자모를 분해한 5-gram near duplicate
- document ID, author ID, source ID의 split 교차
- 중복 case ID와 H1/H2 식별자 불일치

출처 원문은 `scripts/import-evaluation-sources.mjs`로 가져옵니다. 이 단계에서 manifest SHA-256을 확인하고 `manual_authorization` 출처는 다운로드하거나 평가 데이터로 바꾸지 않습니다. training document ID는 H1/H2에 넣을 수 없습니다.

## 품질 게이트

`corpus/gates/commercial-near-v1.json`은 다음을 동시에 요구합니다.

- micro/macro precision, recall, Safe precision의 95% Wilson 하한
- 정상 문장 specificity와 top-1/top-5 교정안 적중률
- 최소 자연 문장·인간 edit·정상 문장·장르·문서·작성자 수
- H1과 H2 각각의 최소 표본
- synthetic 거부와 독립 인간 검수 증거

AI panel 행은 자연 원문과 인간이 작성한 edit의 분모에는 포함될 수 있지만 `independentHumanCases`에는 절대 포함되지 않습니다. 두 holdout과 인간 증거가 실제로 제공되기 전까지 상용 게이트 결과는 NO-GO입니다.

## 운영 경계

모델 API 키나 문서는 이 저장소에 넣지 않습니다. 검수 세션은 로컬에서 실행하고 hash와 최소 메타데이터만 커밋합니다. 원문 라이선스가 재배포를 허용하지 않으면 원문과 예측을 공개 artifact에 포함하지 않고, manifest와 검증 결과만 남깁니다.

# 독립 한국어 정확도 평가 방법

GeulLint의 저장소 회귀 코퍼스와 일반 정확도 주장은 분리한다. `corpus/seed-v1.jsonl`, `corpus/curated-alpha-v1.jsonl`, `corpus/safety-regressions-v1.jsonl`은 구현 회귀와 안전 편집을 검증하는 자료이며 한국어 전반의 상용 수준 정확도를 증명하지 않는다.

## 데이터 계층

평가 입력은 다음 세 계층으로 관리한다.

1. **개발 회귀**: 규칙 작성자가 확인할 수 있는 프로젝트 fixture.
2. **공개 독립 benchmark**: KoLLA 등 제3자 인간 교정 자료와 허용된 문서 revision에서 출처·문서·작성자 단위로 분할한 자료.
3. **release holdout**: 엔진·임계값을 고정한 뒤 실행하는 별도 원문 묶음. 원문은 저장소에 넣지 않고 ID, split, 출처, SHA-256만 manifest로 고정한다.

release-quality 수치는 오류를 합성한 문장으로 만들지 않는다. 합성 오류는 학습과 mutation 회귀 테스트에만 사용한다.

## 필수 표본

상용급 근접 gate는 서로 다른 자연 문장 20,000개 이상을 요구한다. 그 안에 인간 수정 edit 5,000개 이상, 실제 정상 문장 10,000개 이상, 8개 장르, 100개 이상 원문 문서가 있어야 한다. 전문가 다중참조 자료와 revision 자료는 통계를 합치지 않고 각각 표시한다. 홍보할 오류 family는 인간 수정 표본 50개 이상이 없으면 `experimental`로 남긴다.

각 행은 다음 메타데이터를 가진다.

```json
{
  "id": "holdout-news-0001",
  "text": "몇일 뒤에 만나요.",
  "genre": "news",
  "origin": "independent_human",
  "split": "release_holdout",
  "documentId": "sha256:document-001",
  "authorId": "sha256:author-001",
  "errorFamilies": ["spelling"],
  "expectedDiagnostics": [
    {
      "ruleId": "spelling.lexical.myeochil",
      "range": { "start": 0, "end": 6 },
      "suggestions": ["며칠"]
    }
  ]
}
```

`origin`은 `independent_human`, `revision`, `project`, `synthetic` 중 하나다. `independent_human`과 `revision`만 인간 edit 수에 포함하고, `synthetic` 행이 release gate에 들어오면 실패한다. `origin`이 `project`가 아니면 `genre`, `split`, `documentId`가 필수다.

## 누수 차단

문장을 나눈 뒤가 아니라 원문 문서와 작성자를 먼저 분할한다. `node scripts/check-corpus-leakage.mjs --input splits.json`은 다음을 검사한다.

- 분할 사이 정규화 exact text 중복
- 같은 documentId 또는 authorId의 분할 교차
- 자모 5-gram Jaccard 0.85 이상 near duplicate
- 중복 case ID

하나라도 발견되면 종료 코드 1이며 품질 보고서 생성도 중단한다.

## 보고 지표

- 진단 micro/macro precision·recall
- 후보 top-1/top-5 포함률
- 정상 문장 specificity
- 오류 family·장르·origin·split별 지표
- Safe fix precision의 95% Wilson 하한
- calibration ECE
- UTF-8 범위 보존, 수정 멱등성, Native/Web 패리티

자동수정은 점수가 높다는 이유만으로 승격하지 않는다. 독립 holdout에서 family별 Safe gate를 통과하고, 고유명사·신조어·외래어·인용·숫자·문체 교체가 아니며, 수정 후 재검사에서 안정적이어야 한다.

## 출시 규칙

최소 기준은 `corpus/gates/commercial-near-v1.json`에 고정한다. 서로 다른 두 holdout에서 연속 통과하기 전에는 `상용급`, `Harper급`, `네이버급`이라는 비교 문구를 사용하지 않는다. 기준을 못 넘은 family는 Review 또는 Experimental로 강등하고 실패 유형을 다음 학습 세트로만 보낸다.
## AI 주석과 독립 인간 증거의 분리

v2 평가 행은 `textOrigin`과 `annotationOrigin`을 따로 기록한다. `ai_blind_panel`은 모델·rubric·session·output hash와 두 개 이상의 snapshot을 요구하지만 `independentHumanCases`에는 포함되지 않는다. AI 패널의 합의율과 adjudication 비율은 별도 `model-adjudicated-v1` gate로 측정한다. 합의되지 않은 행은 `ambiguous`로 남기며 gold 진단 분모에서 제외한다.

문서·작성자·source ID를 먼저 분할한 뒤 H1/H2를 고정한다. `split-corpus-by-document.mjs`와 `check-corpus-leakage.mjs`가 exact/near duplicate, 자모 5-gram, document·author·source 교차를 검사한다. H1/H2에는 각각 `holdoutId`가 있어야 하며, 두 holdout이 실제 독립 인간 자료로 채워지기 전에는 `commercial-near-v1` 결과를 상용 동급 근거로 사용하지 않는다.

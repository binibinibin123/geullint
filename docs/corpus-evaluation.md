# 코퍼스 평가

GeulLint는 네트워크로 코퍼스를 내려받거나 문서를 외부로 보내지 않습니다. 대신 권한을 가진 사람이 로컬에 둔 JSON Lines 코퍼스를 `geullint --corpus`로 평가합니다. 결과는 진단 단위의 true positive, false positive, false negative, precision, recall을 JSON으로 출력하며, 정상 문장에 대해서는 `normalCases`, `falsePositiveCases`, `specificity`도 별도로 출력합니다. 하나라도 일치하지 않으면 종료 코드 `1`을 반환하므로 CI의 품질 게이트로 바로 쓸 수 있습니다.

```bash
geullint --corpus path/to/gold-corpus.jsonl
```

## 보고서 지표

최상위 `precision`·`recall`은 모든 진단을 합친 micro 지표입니다. `macroPrecision`·`macroRecall`은 코퍼스에서 한 번 이상 기대되거나 실제로 나온 규칙의 정의 가능한 지표를 같은 비중으로 평균낸 값입니다. 양성 기대값이나 실제 진단이 전혀 없어 분모가 0이면 해당 값은 `1.0`이 아니라 JSON `null`입니다. 따라서 전체 카탈로그의 품질을 주장하려면, 먼저 각 출시 대상 규칙이 corpus에 포함됐는지 확인해야 합니다.

`ruleMetrics`는 규칙별 `truePositives`, `falsePositives`, `falseNegatives`, `precision`, `recall`을 안정적인 rule ID 순서로 냅니다. 규칙별 precision 또는 recall도 분모가 0이면 `null`입니다. `precisionWilsonLower95`는 해당 규칙의 관측 precision에 대한 95% Wilson 하한입니다. 실제 진단이 한 번도 없으면 precision 분모가 없으므로 이 필드는 생략됩니다. 작은 표본에서 100%처럼 보이는 규칙을 출시 기준으로 잘못 승격하지 않기 위해 사용합니다.

```json
{
  "precision": 0.99,
  "recall": 0.71,
  "macroPrecision": 0.98,
  "macroRecall": 0.65,
  "ruleMetrics": [
    {
      "ruleId": "spelling.lexical.myeochil",
      "truePositives": 100,
      "falsePositives": 1,
      "falseNegatives": 20,
      "precision": 0.990099,
      "recall": 0.833333,
      "precisionWilsonLower95": 0.946
    }
  ]
}
```

## 출시 품질 게이트

기본 `--corpus`는 한 건의 오탐·누락도 허용하지 않는 fixture 계약입니다. 공개 gold corpus의 통계 기준을 CI에서 판정하려면 로컬 JSON gate를 만들어 `--corpus-gate`로 함께 지정합니다. 이 경우에도 모든 원시 오탐·누락은 `caseFailures`에 남고, 종료 코드는 `qualityGate.passed`로 결정됩니다.

```json
{
  "schemaVersion": 1,
  "minMicroPrecision": 0.98,
  "minMacroPrecision": 0.97,
  "minRecall": 0.65,
  "minRulePrecisionWilsonLower95": 0.95,
  "minExpectedPerRule": 10,
  "requiredRuleIds": [
    "spelling.lexical.myeochil",
    "grammar.conjugation.doe-to-dwae"
  ]
}
```

`requiredRuleIds`는 이번 출시에서 품질을 주장할 규칙의 비어 있지 않은 중복 없는 목록입니다. `minExpectedPerRule`은 이 목록에 든 각 규칙의 최소 양성 표본 수입니다. 표본 수가 모자라거나 그 규칙이 한 번도 진단되지 않아 `precisionWilsonLower95`를 계산할 수 없으면 gate가 실패합니다. 따라서 정상 문장만 든 corpus가 지표 1.0으로 통과하는 일을 막습니다. 모든 비율은 0 이상 1 이하이고, schemaVersion은 현재 `1`만 지원합니다.

```bash
geullint --corpus-manifest gold.manifest.json --corpus-gate release-quality-gate.json
```

gate 결과는 보고서의 `qualityGate`에 `passed`와 실패한 지표·실제값·최소값·해당 `ruleId`를 기록합니다. gate 파일과 corpus 원문은 모두 로컬에서만 읽으며 네트워크·API 키·계정이 필요하지 않습니다.

## JSON Lines v1

한 줄에 한 사례를 넣습니다. 간단한 corpus는 `expectedRuleIds`만 쓰고, gold corpus는 `expectedDiagnostics`로 UTF-8 바이트 범위와 제안까지 정확히 대조합니다. 두 필드는 한 사례에서 함께 쓸 수 없습니다. 빈 줄은 무시합니다.

```json
{"id":"myeochil-001","text":"몇일 뒤에 만나요.","sourceKind":"plain_text","profile":"default","expectedRuleIds":["spelling.lexical.myeochil"]}
```

정확한 annotation 예시는 다음과 같습니다. `range.start`와 `range.end`는 UTF-8 **바이트** 오프셋이며, `suggestions`는 진단이 내놓는 제안 배열과 같은 순서로 일치해야 합니다.

```json
{"id":"myeochil-001-exact","text":"몇일 뒤에 만나요.","expectedDiagnostics":[{"ruleId":"spelling.lexical.myeochil","range":{"start":0,"end":6},"suggestions":["며칠"]}]}
```

| 필드 | 필수 | 설명 |
| --- | --- | --- |
| `id` | 예 | 사례를 식별하는 고유하고 읽기 쉬운 이름 |
| `text` | 예 | UTF-8 검사 문장 또는 코드·문서 조각 |
| `expectedRuleIds` | 아니오 | 기대하는 진단 ID 배열. 같은 ID를 반복해 발생 횟수를 표현하며, `expectedDiagnostics`와 함께 쓸 수 없음 |
| `expectedDiagnostics` | 아니오 | `{ruleId, range?, suggestions?}` 배열. `range` 또는 `suggestions`를 쓴 항목은 해당 값까지 정확히 일치해야 함 |
| `sourceKind` | 아니오 | `plain_text`(기본값), `markdown`, `javascript`, `typescript`, `python`, `rust` |
| `profile` | 아니오 | `default`(기본값), `strict`, `editorial` |

`expectedRuleIds`만 쓴 사례는 규칙 ID와 발생 횟수를 비교합니다. `expectedDiagnostics`를 쓰면 위치·제안도 대조합니다. 수정 후 멱등성과 편집기 UTF-16 위치는 Rust fixture 계약이 별도로 검사합니다.

## 라이선스·출처 manifest

배포하거나 출시 수치에 인용할 외부 코퍼스는 직접 `--corpus`로만 실행하지 말고, 옆에 provenance manifest를 두고 실행합니다. manifest는 로컬 원문 경로·라이선스·원 출처·SHA-256을 묶습니다. CLI는 해시가 맞지 않으면 평가를 시작하지 않고 종료 코드 `2`를 반환합니다.

```json
{
  "schemaVersion": 1,
  "name": "팀이 적법하게 취득한 한국어 오류 교정 코퍼스",
  "license": "권리자가 명시한 라이선스 식별자 또는 전문",
  "sourceUrl": "https://권리자.example/dataset",
  "corpusPath": "gold-corpus.jsonl",
  "sha256": "원문 파일 전체의 소문자 SHA-256 64자리"
}
```

`corpusPath`는 manifest 파일 기준 상대 경로이거나 절대 경로입니다. URL은 정보 기록용일 뿐 CLI가 접근하지 않습니다.

```bash
geullint --corpus-manifest path/to/gold-corpus.manifest.json
```

Windows PowerShell에서 해시를 만들려면 다음 명령을 씁니다.

```powershell
(Get-FileHash -Algorithm SHA256 .\gold-corpus.jsonl).Hash.ToLower()
```

## 저장소의 안전 회귀 코퍼스

[`corpus/safety-regressions-v1.jsonl`](../corpus/safety-regressions-v1.jsonl)은 GeulLint가 직접 작성하고 MIT로 배포하는 적대적 회귀 모음입니다. 오류 72건과 정상 반례 72건을 8개 장르에 각각 18건씩 배치하고 44개 규칙 ID를 다룹니다. plain text뿐 아니라 Markdown, Python, JavaScript, TypeScript, Rust 입력도 포함합니다.

구조 게이트는 [`safety-regressions-v1.policy.json`](../corpus/safety-regressions-v1.policy.json)에 고정되어 있습니다. 정확히 144건인지, 정규화 중복이 없는지, 문자 3-gram 유사도가 한계를 넘지 않는지, 장르·입력 종류·프로필·고위험 규칙 표본이 빠지지 않았는지를 먼저 검사합니다. manifest는 코퍼스 전체 바이트의 SHA-256과 저장소 출처를 고정합니다.

오류 주석은 `original`이 문장에 정확히 한 번 나타나야 합니다. 명시적 `range`가 없으면 평가기가 이 위치에서 UTF-8 바이트 범위를 결정하고 실제 진단의 범위·규칙 ID·제안을 대조합니다. `expectedFixedText`가 원문과 다르면 첫 제안을 적용한 결과와 정확히 같아야 하며, 검토 전용 진단은 원문을 그대로 기대할 수 있습니다. 따라서 범위가 맞아도 엉뚱한 교정문을 만드는 회귀는 출시 게이트를 통과하지 못합니다.

```bash
node scripts/validate-safety-corpus.mjs \
  --corpus corpus/safety-regressions-v1.jsonl \
  --policy corpus/safety-regressions-v1.policy.json \
  --cli target/debug/geullint
target/debug/geullint --corpus-manifest corpus/safety-regressions-v1.manifest.json
```

이 모음은 실제 사용자 문서에서 독립적으로 표본을 뽑고 외부 검토자가 주석한 gold corpus가 아닙니다. 회귀 방지 범위와 현재 구현의 일치 여부만 보여 주며, 한국어 전반의 정밀도·재현율을 증명하지 않습니다.

## 저장소의 seed 코퍼스

[`corpus/seed-v1.jsonl`](../corpus/seed-v1.jsonl)은 현재 공개 규칙 ID가 적어도 한 번은 실제로 실행되는지 확인하는 smoke corpus입니다. 한 문장에서 둘 이상의 진단이 나올 수 있으므로 사례 수와 진단 수는 같지 않을 수 있습니다. 이 파일은 저장소의 MIT 코드와 함께 관리하지만, **정밀도·재현율을 주장할 수 있는 독립 공개 gold corpus가 아닙니다.**

```bash
cargo run -p geullint-cli -- --corpus corpus/seed-v1.jsonl
```

외부 데이터의 원문·사본·계정 토큰·API 키는 이 저장소에 넣지 않습니다. 각 데이터 제공자의 이용 조건에 따라 적법하게 취득한 뒤, 위 manifest와 함께 별도 보관합니다.

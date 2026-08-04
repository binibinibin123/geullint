# 외부 평가 코퍼스 출처

저장소에는 제3자 코퍼스 원문을 포함하지 않습니다. 아래 경로는 외부 파일을 별도 폴더에 취득하고, 라이선스·출처·SHA-256이 남은 manifest로 평가하기 위한 재현 가능한 절차입니다. 검사 런타임은 이 URL에 접근하지 않습니다.

## KoLLA v2 정상 문장 제어군과 검토 대기열

[KoLLA v2.0 Zenodo record](https://zenodo.org/records/16908784)은 한국어 학습자 오류 교정용 다중 참조 M2 데이터를 공개합니다. 현재 record의 `KoLLA_multi-refs.m2`는 `GPL-3.0-or-later` 조건입니다. 따라서 원문을 MIT 저장소·npm 패키지·릴리스 아카이브에 넣지 않고, 해당 조건을 수락한 사용자가 로컬에만 생성합니다.

```powershell
node scripts/acquire-kolla-v2.mjs `
  --accept-gpl-3.0-or-later `
  --out-dir $env:LOCALAPPDATA\geullint\kolla-v2

cargo run -p geullint-cli -- `
  --corpus-manifest $env:LOCALAPPDATA\geullint\kolla-v2\kolla-v2-noop.manifest.json
```

취득 스크립트는 Zenodo가 제공한 M2 파일의 MD5를 먼저 확인한 다음, **모든 주석자가 `noop`으로 판정한 문장만** JSON Lines 정상 제어군으로 정규화합니다. 생성되는 manifest는 그 JSONL의 SHA-256과 GPL-3.0-or-later·Zenodo 출처를 기록합니다. 이 제어군은 기본 프로필의 `falsePositiveCases`와 `specificity`를 측정합니다.

오류·교정 쌍은 같은 로컬 폴더의 `kolla-v2-review-queue.jsonl`에 별도로 만듭니다. 각 행에는 원문 `text`, M2 원문 토큰 `sourceTokens`, 주석자별 `references[].edits[]`의 토큰 범위·오류 범주·교정문이 담깁니다. 이 파일은 `expectedRuleIds`가 없는 **검토 대기열**이므로 `geullint --corpus`에 직접 넣지 않습니다. 사람이 원문과 다중 참조를 검토해 GeulLint rule ID, UTF-8 범위, 제안을 확정한 뒤에만 평가 JSON Lines로 변환합니다.

검토자가 별도 로컬 mapping JSON을 만듭니다. 각 진단은 범위와 제안을 빠짐없이 확정해야 합니다.

```json
{
  "schemaVersion": 1,
  "cases": [
    {
      "reviewId": "kolla-v2-review-1",
      "expectedDiagnostics": [
        {
          "ruleId": "spelling.lexical.myeochil",
          "range": { "start": 0, "end": 6 },
          "suggestions": ["며칠"]
        }
      ],
      "independentReviews": [
        {
          "reviewer": "reviewer-a",
          "expectedDiagnostics": [
            {
              "ruleId": "spelling.lexical.myeochil",
              "range": { "start": 0, "end": 6 },
              "suggestions": ["며칠"]
            }
          ]
        },
        {
          "reviewer": "reviewer-b",
          "expectedDiagnostics": [
            {
              "ruleId": "spelling.lexical.myeochil",
              "range": { "start": 0, "end": 6 },
              "suggestions": ["며칠"]
            }
          ]
        }
      ],
      "adjudicatedBy": "adjudicator-c"
    }
  ]
}
```

[`scripts/curate-kolla-v2-gold.mjs`](../scripts/curate-kolla-v2-gold.mjs)는 mapping의 review ID·UTF-8 바이트 경계를 검증하고 evaluator 형식 JSONL, 일반 corpus manifest, review queue·mapping·출력의 SHA-256이 담긴 별도 provenance를 만듭니다. provenance의 `manifestSha256`이 manifest 전체 바이트를 고정하고, `kolla-v2-curated-gold.provenance.sha256`은 provenance 파일 자체를 고정합니다. `--verify`는 sidecar와 provenance를 먼저 확인한 뒤 manifest의 스키마·corpus 경로·해시와 review queue·mapping·corpus의 실제 바이트 해시를 다시 계산합니다.

정밀도·재현율 같은 품질 수치의 근거로 사용할 corpus는 `--require-independent-review`를 반드시 사용합니다. 각 case에 서로 다른 두 명 이상의 `independentReviews[].reviewer`와, 그 누구와도 다른 `adjudicatedBy`를 기록해야 합니다. reviewer·adjudicator는 외부 계정이나 API 키가 아니라 로컬 검토 기록의 식별자이며, 각 리뷰도 UTF-8 범위와 제안을 갖춘 정확 진단이어야 합니다. 이 옵션은 provenance의 `independentReviewRequired`에도 기록되고, 같은 옵션을 붙인 `--verify`가 그 기록을 확인합니다.

첫 생성에는 존재하지 않는 `--out-dir`을 지정합니다. 스크립트는 같은 부모 폴더의 임시 sibling directory에 네 파일을 모두 쓴 뒤 final directory로 원자적으로 이름을 바꿉니다. final directory가 이미 있거나 중간 단계가 실패하면 기존 directory는 덮어쓰지 않고 임시 directory만 정리하므로, 원인을 해결한 뒤 같은 명령을 다시 실행할 수 있습니다.

```powershell
node scripts/curate-kolla-v2-gold.mjs `
  --review-queue $env:LOCALAPPDATA\geullint\kolla-v2\kolla-v2-review-queue.jsonl `
  --mapping $env:LOCALAPPDATA\geullint\kolla-v2\reviewed-mapping.json `
  --require-independent-review `
  --out-dir $env:LOCALAPPDATA\geullint\kolla-v2\curated

node scripts/curate-kolla-v2-gold.mjs --verify `
  --review-queue $env:LOCALAPPDATA\geullint\kolla-v2\kolla-v2-review-queue.jsonl `
  --mapping $env:LOCALAPPDATA\geullint\kolla-v2\reviewed-mapping.json `
  --require-independent-review `
  --out-dir $env:LOCALAPPDATA\geullint\kolla-v2\curated

cargo run -p geullint-cli -- `
  --corpus-manifest $env:LOCALAPPDATA\geullint\kolla-v2\curated\kolla-v2-curated-gold.manifest.json
```

이 방식은 현재 Zenodo 원문이 제공하는 다중 주석을 임의로 “정답 오류”로 바꾸지 않기 위한 선택입니다. 생성 파일은 Git이 아닌 사용자가 지정한 외부 폴더에만 두세요.

## 요청이 필요한 국립국어원 코퍼스

국립국어원 말뭉치 등록소의 [맞춤법 교정 말뭉치 2021·2022](https://kli.korean.go.kr/request/corpusRegist.do?lang=en)는 웹 텍스트의 맞춤법 오류 교정 자료로 소개되지만, 현재 등록소에서 `Request` 절차를 거칩니다. 권한을 받은 뒤에만 원문을 내려받고, [코퍼스 평가 형식](corpus-evaluation.md)의 JSON Lines와 manifest로 변환해야 합니다. 요청 전·권한 없이 원문을 미러링하거나 저장소에 올리지 않습니다.
### Authorization-only release holdout

`data/sources.json` records the NIKL spelling-correction corpus as
`access: "manual_authorization"`, `redistributable: false`, and `sha256: null`.
The acquisition tool validates that entry but never downloads the request page or
counts it as evaluation data. After the corpus owner grants access, the local
manifest must be replaced with the exact authorized artifact hash and a provenance
record before `release_holdout` or `independent_human` cases can enter a quality gate.

# v0.2.0-alpha.1 품질 보고서

이 보고서는 GeulLint의 현재 검증 범위와 한계를 함께 공개합니다. 아래 수치는 네이버 맞춤법 검사기나 다른 서비스와 비교하는 정확도 수치가 아닙니다.

## 공개 카탈로그

- 공개 규칙: 100개
- 구성: Rust 문맥 규칙 25개, 기존 수동 카탈로그 33개, 새로 검수한 철자 규칙 42개
- 기본 활성화: 97개
- 검토 전용: 3개
- 카테고리: 철자 67, 문법 13, 띄어쓰기 10, 문장부호 4, 반복 2, 문체 2, 기술 용어 1, 고급 제안 1

`100`은 품질 목표가 아니라 이번 알파의 상한입니다. 오탐이 발견된 규칙은 수를 유지하기 위해 다른 규칙으로 채우지 않습니다.

## 문장 단위 내부 회귀

[`corpus/curated-alpha-v1.jsonl`](../corpus/curated-alpha-v1.jsonl)은 새 철자 규칙 42개를 다음 문장으로 검사합니다.

- 서로 다른 오류 문장: 84개
- 정상 반례: 42개
- 총 문장: 126개
- 평가 결과: TP 84, FP 0, FN 0

이 자료는 프로젝트가 직접 작성한 회귀 자료이므로 독립 정확도 평가가 아닙니다.

```bash
cargo run -p geullint-cli -- --corpus corpus/curated-alpha-v1.jsonl
```

## 외부 정상 문장 오탐 감사

[KoLLA v2.0](https://zenodo.org/records/16908784)의 다중 주석자가 모두 수정 불필요로 판정한 문장만 로컬에서 추출해 기본 프로필의 오탐을 확인했습니다. 원문은 GPL-3.0-or-later이므로 MIT 저장소와 릴리스에는 포함하지 않습니다.

| 항목 | 값 |
| --- | ---: |
| 정상 문장 | 249 |
| 오탐이 발생한 문장 | 0 |
| 진단 수 | 0 |
| specificity | 1.0 |

양성 표본이 없는 정상 제어군이므로 precision·recall·macro 지표는 JSON `null`이며 정확도 근거로 사용하지 않습니다.

재현 식별자:

- KoLLA 원본 MD5: `9a6f2e3fea1b39bbb7343445db1167f7`
- 정규화한 정상 제어군 SHA-256: `f1c14d9ab8f0bc21945b6652d31efe1b722a4837fb86cfc8f084deb7409b7b01`
- 라이선스: `GPL-3.0-or-later`

```powershell
$audit = Join-Path $env:LOCALAPPDATA "geullint\kolla-v2"
node scripts/acquire-kolla-v2.mjs --accept-gpl-3.0-or-later --out-dir $audit
cargo run -p geullint-cli -- --corpus-manifest `
  (Join-Path $audit "kolla-v2-noop.manifest.json")
```

## 주장하지 않는 것

- 249개 정상 문장은 모든 장르·연령·작성자를 대표하지 않습니다.
- 오류가 포함된 독립 gold corpus를 아직 이중 검토하지 않았습니다.
- 따라서 독립 precision, recall, 네이버 대비 정확도를 주장하지 않습니다.
- matcher contract와 100개 smoke corpus는 로딩·범위·수정 멱등성 검사이며 언어 품질 사례로 세지 않습니다.

다음 품질 단계는 KoLLA 교정 대기열과 허가받은 국립국어원 자료를 서로 다른 두 명이 검토하고, 별도 판정자가 확정한 오류·정상 혼합 gold corpus를 만드는 것입니다.

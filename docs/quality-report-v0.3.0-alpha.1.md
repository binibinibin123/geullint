# v0.3.0-alpha.1 품질 보고서

이 보고서는 GeulLint가 이번 알파에서 실제로 검증한 범위와 아직 검증하지 못한 범위를 함께 기록합니다. 아래 수치는 네이버 맞춤법 검사기나 Harper를 상대로 한 비교 정확도가 아닙니다.

## 공개 카탈로그

- 공개 규칙: 113개
- 기본 활성화: 102개
- 안전 자동 수정: 94개
- 검토가 필요한 제안: 19개
- 카테고리: 철자 67, 문법 19, 띄어쓰기 17, 문장부호 4, 반복 2, 문체 2, 기술 용어 1, 고급 제안 1

`113`은 품질 점수가 아닙니다. 오탐을 막을 정상 반례가 부족한 후보는 카탈로그 수를 늘리기 위해 넣지 않았습니다.

이번 버전은 `-읍니다/-읍니까`, `-십시요`, `아니예요`, `않되-/않돼-`와 일곱 의존 명사 계열을 문맥 규칙으로 추가했습니다. 이 11개 규칙은 서로 다른 오류 문장 33개, 정상·고유명사·어휘 반례 56개, 코드 문자열 제외와 주석 검사 22개를 테스트합니다. 모든 사례에서 UTF-8 범위와 수정 후 멱등성도 확인합니다.

## 프로젝트 소유 출시 안전 회귀

[`safety-regressions-v1.jsonl`](../corpus/safety-regressions-v1.jsonl)은 같은 문장을 숫자만 바꿔 늘리지 않은 144개 문장으로 구성됩니다.

| 항목 | 값 |
| --- | ---: |
| 오류 문장 | 72 |
| 정상 문장 | 72 |
| 장르 | 8 |
| source kind | 6 |
| 평가된 규칙 ID | 44 |
| TP / FP / FN | 73 / 0 / 0 |

- corpus SHA-256: `9f87c5c0a09a2d406e168c280039d14b8729b6ce1b82a0bb352a07b0f3df601c`
- 구조 정책과 manifest가 corpus의 해시, 중복, 장르 균형, 정확한 규칙 ID·범위·제안·교정문을 출시 때 다시 검사합니다.

이 모음은 프로젝트가 직접 작성한 적대적 회귀 자료입니다. 여기서 FP와 FN이 0이라는 사실은 등록된 사례가 다시 깨지지 않았다는 뜻이며, 일반 한국어 정확도가 100%라는 뜻이 아닙니다.

```bash
node scripts/validate-safety-corpus.mjs \
  --corpus corpus/safety-regressions-v1.jsonl \
  --policy corpus/safety-regressions-v1.policy.json \
  --cli target/debug/geullint
target/debug/geullint --corpus-manifest corpus/safety-regressions-v1.manifest.json
```

## 검수 철자 회귀

[`curated-alpha-v1.jsonl`](../corpus/curated-alpha-v1.jsonl)은 검수 철자 규칙 42개를 서로 다른 오류 문장 84개와 정상 반례 42개로 검사합니다.

| 항목 | 값 |
| --- | ---: |
| 전체 문장 | 126 |
| TP / FP / FN | 84 / 0 / 0 |
| corpus SHA-256 | `c09c2575f0444abb65fa1c30fef8a6a7d4939e5fc5b1067474ec6c42cd57e00e` |

이 자료도 프로젝트 소유 회귀이므로 독립 정확도 평가로 사용하지 않습니다.

```bash
target/debug/geullint --corpus corpus/curated-alpha-v1.jsonl
```

## 외부 정상 문장 오탐 감사

[KoLLA v2.0](https://zenodo.org/records/16908784)의 다중 주석자가 모두 수정 불필요로 판정한 문장만 로컬에서 추출해 기본 프로필의 오탐을 확인했습니다. 원문은 GPL-3.0-or-later이므로 MIT 저장소와 릴리스에 포함하지 않습니다.

| 항목 | 값 |
| --- | ---: |
| 정상 문장 | 249 |
| 오탐 문장 / 진단 | 0 / 0 |
| specificity | 1.0 |

- KoLLA 원본 MD5: `9a6f2e3fea1b39bbb7343445db1167f7`
- 정규화 제어군 SHA-256: `f1c14d9ab8f0bc21945b6652d31efe1b722a4837fb86cfc8f084deb7409b7b01`

양성 표본이 없는 정상 제어군이므로 precision·recall은 계산하지 않습니다. 249문장은 모든 장르·연령·작성자를 대표하지 않습니다.

## 범용 철자 사전 실험

[Hunspell](https://github.com/hunspell/hunspell)과 [한국어 사전 0.7.94](https://github.com/spellcheck-ko/hunspell-dict-ko)를 별도 환경에서 Native와 WASM으로 평가했지만 이번 릴리스에는 넣지 않았습니다.

- KoLLA no-op 249문장 중 46문장(18.5%)에서 하나 이상의 어절을 OOV로 거부했습니다.
- KoLLA의 두 주석자가 합의한 단일 어절 오류 span 597개 중 404개(67.67%)를 거부했습니다.
- 거부된 오류에서 균등 추출한 40개 제안 표본은 gold 교정형 top-1 10개(25%), top-5 16개(40%)였습니다.
- 기존 WASM 포트는 사전 생성에 2.12–2.37초, 제안에 p50 355 ms·p95 1.103초가 걸렸고 메모리 사용량이 약 54–56 MB 늘었습니다.

이 결과는 탐지 가능성은 보여 주지만 기본 진단으로 내보내기에는 오탐 후보와 제안 품질이 부족합니다. 오래된 WASM 포트 대신 최신 Hunspell을 직접 빌드하고, 고유명사 허용 목록·비동기 제안·독립 오류/정상 gold gate를 갖춘 뒤 `review-only` 선택 기능으로 다시 검토합니다. 사용자의 문장을 과도하게 지적하는 사전을 규칙 수나 기능 수를 늘리기 위해 기본 배포에 포함하지 않습니다.

## 소스 경계와 표면 일치

Native와 web-target WASM 산출물은 같은 43개 사례를 통과합니다. Markdown 코드·링크·자동 링크·이미지 주소, 일반 텍스트의 URL·전자우편·파일 경로·파일명·도메인·해시태그와 뒤따르는 한국어 조사, JavaScript·TypeScript·Python·Rust의 코드·문자열은 건드리지 않고 검사 가능한 본문과 주석만 같은 UTF-8 범위로 반환합니다.

- source parity fixture SHA-256: `dc28e05d96be5a4b319a89ce9a1fe58364cf6a298bc8f6eb08d317ba9224fae3`
- 브라우저 산출물: WASM 498,703 B raw / 180,252 B gzip
- 상세 시간과 재현 명령: [성능 측정](performance.md)

## 현재 주장하지 않는 것

- 아직 임의의 미등록 단어를 판정하고 후보를 생성하는 범용 OOV 철자 사전은 기본 엔진에 없습니다. 등록된 철자·활용·띄어쓰기·문법 규칙 밖의 오타는 놓칠 수 있습니다.
- 오류가 섞인 외부 gold corpus를 두 명 이상이 독립 검토하지 않았으므로 외부 precision·recall이나 다른 검사기 대비 정확도를 주장하지 않습니다.
- matcher contract와 규칙별 smoke corpus는 배선·범위·멱등성 검사이지 실세계 언어 품질 표본이 아닙니다.
- 의미가 달라질 수 있는 의존 명사·말투 제안은 자동 수정하지 않습니다.

따라서 v0.3.0-alpha.1의 공개 위치는 **빠르고 보수적인 오프라인 한국어 규칙 검사기 알파**입니다. 범용 철자 후보 생성과 독립 오류 gold corpus가 갖춰지기 전에는 Harper나 상용 맞춤법 검사기와 동급이라고 부르지 않습니다.

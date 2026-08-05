# 공개 데이터 실제 평가 v1

이 문서는 규칙 수가 아니라 실제 문장으로 GeulLint를 측정한 기록이다. 원문은 저장소에 재배포하지 않고, 다운로드·해시·라이선스 확인 후 로컬에서만 평가했다.

## 실행한 데이터

| 출처 | 용도 | 라이선스/조건 | 사용 건수 |
| --- | --- | --- | ---: |
| [Tatoeba Korean export](https://tatoeba.org/en/downloads) | 정상 문장 | CC BY 2.0 | 15,936 |
| [K-NCT](https://github.com/seonminkoo/K-NCT) | 오류-교정 쌍 | 저장소에 재배포 라이선스가 없어 로컬 평가 전용 | 2,999 |
| [KoLLA v2](https://zenodo.org/records/16908784) | 다중 교정 참고자료 | GPL-3.0-or-later | 1,162 |
| GeulLint 안전 회귀 모음 | 기존 회귀 | 프로젝트 라이선스 | 144 |
| Tatoeba에서 결정적으로 만든 spacing 변형 | 합성 오류(인간 검수 아님) | Tatoeba 원문에서 생성 | 1,000 |

중복 문장을 다른 split에 두지 않도록 11건을 제거했다. 최종 번들은 21,169건(정상 15,936, 오류 5,233)이며 `train 16,864 / H1 2,999 / H2 1,162`로 나뉜다. H1/H2는 문서·작성자·출처 ID와 정규화 텍스트 및 분해 자모 5-gram 누수를 검사했다.

최종 JSONL SHA-256은 `4c587caf77200161f15d013aa05f5e18805a742c218edbca54ee6ab749cb49f9`이며, 요약 수치는 [public-evaluation-v1-summary.json](public-evaluation-v1-summary.json)에 고정했다. 누수 검사는 최종 번들에서 통과했다.

재현 명령(원문을 먼저 내려받아야 함):

```powershell
node scripts/build-public-evaluation-bundle.mjs `
  --tatoeba PATH\kor_sentences.tsv `
  --tatoeba-source PATH\kor_sentences.tsv.bz2 `
  --knct PATH\K-NCT_v1.4.json `
  --kolla PATH\kolla-v2-review-queue.jsonl `
  --kolla-source PATH\KoLLA_multi-refs.m2 `
  --synthetic-corrections 1000 `
  --safety corpus\safety-regressions-v1.jsonl `
  --out-dir OUT\public-bundle-v1

node scripts/check-corpus-leakage.mjs --input OUT\public-bundle-v1\public-evaluation-v1.leakage.json
geullint --corpus OUT\public-bundle-v1\public-evaluation-v1.jsonl --format json > OUT\public-bundle-v1\native-report.json
node scripts/summarize-public-evaluation.mjs `
  --corpus OUT\public-bundle-v1\public-evaluation-v1.jsonl `
  --native-report OUT\public-bundle-v1\native-report.json `
  --out OUT\public-bundle-v1\summary.json
node scripts/build-playground.mjs
node scripts/wasm-runtime-parity.mjs --report OUT\public-bundle-v1\wasm-parity-report.json
```

## 실제 결과

| 지표 | 결과 |
| --- | ---: |
| Native 정상 문장 specificity (오탐 없는 케이스 비율) | **99.567%** |
| Native 전체 precision | 38.421% |
| Native 전체 recall | 1.395% |
| Native top-1 / top-5 교정 정확도 | 1.395% / 1.395% |
| 공개 source-revision의 원문 일치 교정률 | **0.264% (11/4,161)** |
| 독립 인간 holdout | **0건** |
| AI blind-panel/adjudication | **0건** |
| WASM source-parity | 43/43 통과 |

결론은 **상용 품질 게이트 NO-GO**다. 정상 문장에 대한 안전성은 높지만, 공개 오류-교정 자료에서 실제 교정률이 낮고 독립 인간 검수가 아직 없다. 합성 1,000건은 분량을 늘리기 위한 별도 스트레스 세트일 뿐 인간 교정 자료나 AI 검수로 세지 않는다. 따라서 이 결과를 “네이버급” 또는 “HARPER급”이라고 홍보해서는 안 된다. 다음 릴리스의 필수 작업은 (1) 재배포 가능한 5,000건 이상 인간 교정 자료 확보, (2) H2 2,500건 이상 문서/작성자 분리, (3) 실제 독립 인간 주석 holdout, (4) 그 뒤 규칙·형태소·교정 랭커 개선이다.

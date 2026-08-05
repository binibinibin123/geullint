# 공개 평가 번들 v2

이 문서는 저장소에 원문 코퍼스를 커밋하지 않고, 외부에서 다시 내려받아 재현할 수 있는 평가 절차와 실제 결과를 기록합니다. 현재 결과는 상용 게이트를 통과하지 않았습니다.

## 번들 구성

| 구분 | 건수 | 출처 및 주의 |
| --- | ---: | --- |
| 정상 문장 | 16,538 | Tatoeba 15,866 + KWikiText 600 + 안전 회귀 72 |
| 교정 문장 | 5,084 | K-NCT 2,999, KoLLA 2,013, 안전 회귀 72 |
| 전체 | 21,622 | 합성 교정 0건 |
| 독립 인간 주석 | 1,688 | KoLLA 다중 참고자 844행의 2개 참고자 |
| H1 / H2 | 2,999 / 2,613 | 서로 다른 문서·작성자·source ID로 분리 |

작성자 메타데이터를 포함한 Tatoeba 상세 내보내기에서 235명의 실제 작성자와 20,634개의 문서를 확인했습니다. K-NCT는 재배포 금지 조건 때문에 로컬 평가 전용이며, 원문은 저장소에 포함하지 않습니다.

출처:

- [Tatoeba downloads](https://tatoeba.org/en/downloads) — CC BY 2.0
- [K-NCT](https://github.com/seonminkoo/K-NCT) — 저장소에 명시된 재배포 라이선스가 없어 로컬 전용
- [KoLLA v2](https://zenodo.org/records/16908784) — GPL-3.0-or-later
- [KWikiText 2020 test](https://github.com/lovit/kowikitext/releases/tag/20200920.v1) — CC BY-SA 3.0

최종 번들 SHA-256:

`be55575da5701192b60ca7566db9c2414919a435f0142c303ba5fc6cbd564114`

## 검증 결과

| 지표 | 결과 |
| --- | ---: |
| 분할 누수 검사 | 통과 (21,622건, exact·near duplicate·문서·작성자·source·자모 5-gram) |
| Native/WASM parity | 통과 (43건) |
| Native specificity | 99.389% |
| Native precision | 27.444% |
| Native recall | 1.436% |
| top-1 / top-5 교정 정확도 | 1.436% / 1.436% |
| 독립 인간 주석 exact 교정 | 6 / 1,688 (0.355%) |

상용 게이트는 다음 이유로 **NO-GO**입니다.

- precision 0.274, recall 0.014로 게이트 기준(0.98 / 0.85)에 미달
- 필수 규칙 3개의 평가 사례 수가 50건 미만
- 별도 `release_holdout`이 아직 없음

따라서 이 번들은 데이터·누수·출처 검증을 위한 공개 평가용이며, “상용급”, “Harper급”, “네이버급”이라는 성능 주장을 뒷받침하지 않습니다.

## 재현 명령

```powershell
node scripts/build-public-evaluation-bundle.mjs `
  --tatoeba PATH\kor_sentences.tsv `
  --tatoeba-source PATH\kor_sentences.tsv.bz2 `
  --tatoeba-detailed PATH\sentences_detailed_kor.tsv `
  --knct PATH\K-NCT_v1.4.json `
  --kolla PATH\kolla-v2-review-queue.jsonl `
  --kolla-source PATH\KoLLA_multi-refs.m2 `
  --kowikitext PATH\kowikitext_20200920.test `
  --kowikitext-source PATH\kowikitext_test.zip `
  --safety corpus\safety-regressions-v1.jsonl `
  --out-dir OUT\public-bundle-v2

node scripts/check-corpus-leakage.mjs `
  --input OUT\public-bundle-v2\public-evaluation-v1.leakage.json

geullint --corpus OUT\public-bundle-v2\public-evaluation-v1.jsonl `
  --format json > OUT\public-bundle-v2\native-report.json

node scripts/summarize-public-evaluation.mjs `
  --corpus OUT\public-bundle-v2\public-evaluation-v1.jsonl `
  --native-report OUT\public-bundle-v2\native-report.json `
  --out OUT\public-bundle-v2\summary.json
```

AI blind-panel 결과는 별도 [`ai-review-v1-report.json`](ai-review-v1-report.json)과 `model-adjudicated-v1` 개발 게이트로 보관합니다. AI 결과는 인간 증거로 승격하지 않으며, 모호하거나 충돌한 행은 adjudication 후에도 상용 holdout 지표에 자동 포함하지 않습니다.

# GeulLint v0.4.0-beta.1 품질 보고서 초안

이 문서는 상용 검사기와의 우열을 주장하는 문서가 아니라, 베타 릴리스 전에 반복 실행할 품질 계약과 현재 증거를 분리해 기록한다.

## 현재 확인된 것

- Rust workspace 전체 테스트와 WASM/native source-parity fixture가 통과한다.
- 안전 회귀 코퍼스는 144개 사례(오류 72, 정상 72)로 고정되어 있다.
- CLI는 사람용·JSON·SARIF 출력, `stdin`, 변경 파일, watch, 원자적 `--fix`, 내용 해시 캐시를 제공한다.
- 브라우저는 Web Worker·WASM·IndexedDB를 사용하며 검사 중 외부 네트워크 요청을 만들지 않는다.

## 아직 베타 GO가 아닌 항목

독립 자연 문장 20,000개, 인간 수정 5,000개, 정상 10,000개, 8개 장르와 두 개 holdout을 확보하고 검토하기 전에는 `commercial-near-v1` 게이트를 통과했다고 말하지 않는다. 현재 저장소의 작은 회귀 코퍼스와 공개 데이터 취득 결과만으로 일반 정밀도·재현율을 산출하지 않는다.

게이트 재현 명령:

```bash
cargo build -p geullint-cli
node scripts/check-corpus-leakage.mjs --input path/to/corpora.json
node scripts/evaluate-commercial-gate.mjs \
  --cli target/debug/geullint \
  --corpus path/to/release-holdout.jsonl \
  --gate corpus/gates/commercial-near-v1.json
node scripts/red-team-korean.mjs --cli target/debug/geullint
```

게이트 실패는 출시를 막는 정상 동작이다. 실패한 오류 family는 holdout 문장 자체를 튜닝 자료로 되돌리지 않고 새 dev 표본으로 재현한다.

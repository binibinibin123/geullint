# GeulLint 성능 측정

이 문서는 재현 가능한 개발 기준선을 공개합니다. 생성 입력은 처리량 측정용이며 맞춤법 정확도 자료가 아닙니다. 진단 수 역시 실행 경로 확인값일 뿐 precision·recall·오탐률을 뜻하지 않습니다.

## 측정 조건

- 날짜: 2026-08-02
- 기준 커밋: `fface5df1efec24e8ca5270710e8b86d7bbfe9c2`
- 운영체제: Windows 11 Pro 10.0.22631, x64
- CPU: Intel Core i7-8700 3.20 GHz
- 메모리: 63.9 GiB
- Rust: `rustc 1.96.0 (ac68faa20 2026-05-25)`
- Node.js: `v25.3.0`
- 빌드: Cargo `release`, WebAssembly는 `wasm-bindgen 0.2.126`의 `web` target
- 표본: fixture마다 warmup 3회 뒤 20회 측정

CPU 고정이나 전용 성능 장비를 사용하지 않은 단일 로컬 측정입니다. 절대 성능 보장이나 다른 제품과의 비교가 아니라 같은 runner에서 회귀를 찾기 위한 기준선입니다.

## 입력

`scripts/benchmark-fixtures.mjs`가 일반 텍스트·Markdown·TypeScript 각각에 대해 1 KiB, 100 KiB, 1 MiB 입력을 결정적으로 만듭니다. TypeScript 입력에는 실행 코드·문자열·줄 주석·블록 주석이 섞여 있습니다.

1 MiB 입력의 SHA-256은 다음과 같습니다.

| 입력 | SHA-256 |
| --- | --- |
| plain | `22943a8e36e31b333430dcb73610754269dfe373a812d652b7a3d21d68c13d48` |
| markdown | `d40754470dc01f75d1eb6a99efc3fad0ae01810f8df1d726a8f249ee90ea8c04` |
| typescript | `c127304c601bee00ea633d1de5b9b5df7bfc44992ccb7c0a75ac0f44f1d89c03` |

runner의 JSON 결과에는 아홉 입력 모두의 byte 크기와 SHA-256이 기록됩니다.

## Warm 검사 시간

단위는 밀리초이며 `p50 / p95` 순서입니다. Native는 기본 릴리스 빌드입니다. 가벼운 소스 스캐너가 항상 포함되므로 과거의 `compact`와 `source` 모드는 현재 같은 기능 집합을 빌드합니다. WASM 수치는 `lint_json` 호출, Rust 응답 직렬화, Node의 JSON 해석을 포함합니다. 대용량 검사 기준선은 진단과 안전 교정문을 계산하며, 선택적인 검토 교정문은 `includeReviewFixes: false`로 제외합니다.

| 입력 | Native | WASM | 진단 수 |
| --- | ---: | ---: | ---: |
| plain 1 KiB | 0.238 / 0.323 | 0.994 / 1.869 | 0 |
| plain 100 KiB | 26.593 / 31.516 | 38.294 / 42.859 | 53 |
| plain 1 MiB | 246.847 / 274.208 | 376.395 / 398.330 | 540 |
| markdown 1 KiB | 0.244 / 0.316 | 0.494 / 0.845 | 0 |
| markdown 100 KiB | 29.078 / 33.305 | 45.484 / 51.184 | 49 |
| markdown 1 MiB | 297.631 / 326.079 | 467.653 / 479.391 | 502 |
| typescript 1 KiB | 0.141 / 0.171 | 0.389 / 0.516 | 0 |
| typescript 100 KiB | 18.314 / 20.983 | 29.073 / 33.794 | 36 |
| typescript 1 MiB | 188.498 / 214.715 | 295.574 / 323.853 | 374 |

Native runner는 fixture마다 새 probe 프로세스를 실행합니다. 첫 검사는 전역 matcher 초기화를 포함했으며 1 KiB 입력에서 6.507–6.663 ms, 1 MiB 입력에서 188.735–294.325 ms였습니다. WASM compile/instantiate는 모듈 파일을 읽은 뒤 49.547 ms였습니다. 네트워크 전송, 압축 해제, 브라우저 worker 생성은 포함하지 않습니다.

## 산출물 크기

`node scripts/artifact-budgets.mjs`는 raw와 gzip level 9 크기를 모두 검사합니다. 상한은 기능 추가 여지를 두면서 대형 사전이나 의존성이 웹 번들에 실수로 들어가는 일을 막도록 정했습니다.

| 산출물 | 실제 raw | raw 상한 | 실제 gzip | gzip 상한 |
| --- | ---: | ---: | ---: | ---: |
| `geullint_wasm_bg.wasm` | 621,434 B | 650,000 B | 212,557 B | 220,000 B |
| `geullint_wasm.js` | 11,733 B | 14,000 B | 2,632 B | 2,900 B |

시간값은 공유 CI 환경에서 흔들리므로 자동 실패 기준이 아닙니다. byte 크기는 결정적이므로 상한을 넘으면 실패합니다.

## 재현

```powershell
node scripts/benchmark-native.mjs --mode source --warmup 3 --iterations 20
node scripts/build-playground.mjs
node scripts/benchmark-wasm.mjs --warmup 3 --iterations 20
node scripts/artifact-budgets.mjs
```

`--mode compact`는 이전 실행 명령과의 호환 별칭입니다. 형태소 API를 포함한 큰 opt-in 빌드는 `--mode morphology`로 따로 측정할 수 있습니다. 기본 릴리스와 브라우저에는 형태소 사전이 포함되지 않습니다.
The browser artifact measurements above are from the `standard` feature build used by the
playground selector. The compact API remains available for small embedders; standard and
context candidate suggestions are Review-only and do not change the Safe-fix budget.

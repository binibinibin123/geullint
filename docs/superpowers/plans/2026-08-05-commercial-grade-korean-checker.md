# Commercial-Grade Korean Checker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** GeulLint를 사용자의 데이터 제공·수동 검수·API 발급·배포 작업 없이 독립 실문장 지표로 검증된 고정밀 오프라인 한국어 검사기로 만든다.

**Architecture:** 기존 결정론적 Rust 규칙 엔진을 형태소 격자, 압축 어휘 사전, 제한된 철자·띄어쓰기 후보 생성, 10~15M parameter INT8 문맥 랭커, Safe/Review/Abstain 정책으로 확장한다. compact 엔진은 유지하고 standard 엔진을 웹·VS Code·CLI의 기본 사용자 제품으로 제공한다.

**Tech Stack:** Rust 2024, Lindera/mecab-ko-dic, FST/DAWG, Python/PyTorch 학습, ONNX INT8, WebAssembly/Web Worker/Service Worker, TypeScript VS Code extension, Node.js 데이터·릴리스 검증, GitHub Actions/Pages/Releases.

---

## 프로그램 원칙

- 규칙 개수는 목표나 품질 증거로 사용하지 않는다.
- 합성 오류는 학습·회귀 전용이며 공개 정확도 평가는 자연 발생 인간 교정 자료로 한다.
- 사용자 계정, API 키, 수동 다운로드, 사용자 라벨링, 유료 서비스는 필수 경로에서 제외한다.
- 모델은 후보의 순위만 정하고 자유 형식 교정문을 생성하지 않는다.
- 품질 게이트를 못 넘긴 진단은 자동으로 Review, Experimental 또는 Abstain으로 강등한다.
- 각 단계는 별도 PR과 작은 목적별 커밋으로 완료하며 공개 이력을 재작성하지 않는다.

## 파일 구조 지도

### 기존 파일 분리·수정

- `crates/geullint-core/src/lib.rs`: 공개 API re-export만 남기고 87KB 단일 구현을 모듈로 분리한다.
- `crates/geullint-core/src/analysis.rs`: 분석 backend 계약과 형태소 격자 진입점으로 바꾼다.
- `crates/geullint-core/src/productive.rs`: 의존명사·조사·활용 규칙을 전용 후보 모듈로 이관한다.
- `crates/geullint-core/src/endings.rs`: 원형·어미 기반 후보 생성으로 이관한다.
- `crates/geullint-cli/src/main.rs`: v2 진단, 확장 corpus gate, standard 자산, 새 명령을 연결한다.
- `crates/geullint-lsp/src/lib.rs`: 증분 동기화, 문장 캐시, 다중 제안과 새 Code Action을 연결한다.
- `crates/geullint-wasm/src/lib.rs`: standard 자산 로딩과 v2 진단 계약을 노출한다.
- `apps/playground/worker.js`: 모델·사전 초기화, 취소, 캐시 상태와 검사 요청을 처리한다.
- `apps/playground/app.js`, `apps/playground/app.css`, `apps/playground/index.html`: diff, 적용·거절, undo/redo, 사전, 파일 입출력, PWA 상태를 구현한다.
- `extensions/vscode-geullint/src/extension.ts`: Fix All, preview, ignore, dictionary, 상태 표시를 구현한다.
- `.github/workflows/ci.yml`, `.github/workflows/release.yml`: 데이터·모델·패리티·품질·공급망 gate를 연결한다.

### 새 엔진 모듈

- `crates/geullint-core/src/api.rs`: `DiagnosticV2`, `Suggestion`, `Evidence`, `FixSafety` 계약.
- `crates/geullint-core/src/pipeline.rs`: 분석부터 편집 계획까지의 실행 순서.
- `crates/geullint-core/src/analysis/lattice.rs`: 복수 형태소 경로와 UTF-8 범위.
- `crates/geullint-core/src/analysis/phonology.rs`: 종성, ㄹ 예외, 자모와 음운 feature.
- `crates/geullint-core/src/lexicon/mod.rs`: versioned FST/DAWG 로더와 사용자 사전 우선순위.
- `crates/geullint-core/src/candidate/spelling.rs`: 한글 가중 편집 후보.
- `crates/geullint-core/src/candidate/spacing.rs`: 2~4어절 결합·분리 후보.
- `crates/geullint-core/src/candidate/grammar.rs`: 활용, 조사, 어미 후보.
- `crates/geullint-core/src/ranking/mod.rs`: scorer trait와 결정론적 1차 필터.
- `crates/geullint-core/src/policy.rs`: 유형별 Safe/Review/Abstain 결정.
- `crates/geullint-core/src/planner.rs`: 겹침 해소와 멱등 편집 계획.
- `crates/geullint-core/src/trace.rs`: 설명 가능한 근거 기록.

### 새 데이터·모델·평가 파일

- `data/sources.json`: 자동 취득 가능한 자료의 URL, 라이선스, 버전, 해시.
- `scripts/acquire-training-data.mjs`: 허용 출처만 내려받는 재현 취득기.
- `scripts/extract-human-edits.mjs`: 허용된 revision과 commit에서 국소 수정쌍 추출.
- `scripts/check-corpus-leakage.mjs`: exact, normalized, 자모 MinHash, 문서 중복 차단.
- `scripts/build-standard-lexicon.mjs`: 사전·빈도·출처를 FST/DAWG 자산으로 컴파일.
- `training/pyproject.toml`: 고정된 학습·평가 의존성.
- `training/geullint_training/build_pairs.py`: 실오류·합성오류·hard negative 학습쌍.
- `training/geullint_training/train_ranker.py`: GeulRank-small 재현 학습.
- `training/geullint_training/export_onnx.py`: INT8 ONNX와 calibration 산출물.
- `models/geulrank-small/manifest.json`: 모델 해시, 크기, vocabulary, 품질, 라이선스.
- `models/geulrank-small/MODEL_CARD.md`: 학습 출처, 용도, 한계, 편향.
- `corpus/gates/commercial-near-v1.json`: 출시 정량 기준.
- `docs/accuracy-methodology.md`: 독립 평가, 누수 방지, confidence interval.
- `docs/data-card.md`: 데이터 출처와 제외 기준.

## 실행 순서

> **진행 기록 (2026-08-05):** Task 1의 JSONL 메타데이터, dataset quality gate, 장르·출처 slice 집계, exact/document/author/5-gram 누수 검사와 회귀 테스트는 구현했다. 독립 자연 문장 풀을 실제로 취득해 20,000건·인간 edit 5,000건 기준을 충족하는 단계는 아직 남아 있다.

> **구현 기록 (2026-08-05, 현재 브랜치):** API·pipeline 경계, 후보 생성기·안전 정책, 증분 LSP, 오프라인 PWA 저장/피드백, CLI의 `init`·`doctor`·`--stdin`·`--changed`·`--watch`·원자적 `fix`·내용 해시 캐시·completion, 표준 사전 자산, portable ranker 계약, red-team/품질 보고서와 CI/release 검증 연결까지 구현했다. `--changed`는 staged·working tree·untracked 파일을 모두 검사하며 CLI에는 평문 출력 계약인 `--no-color`가 있다. 전체 Rust·Node·Python 게이트는 통과한다.
>
> **아직 닫히지 않은 출구조건:** 외부 독립 인간 교정 5,000 edit와 20,000 자연 문장·두 개 release holdout, 실제 학습된 INT8 ONNX 모델, Firefox/WebKit·모바일을 포함한 전체 Native/Web E2E와 6개 runner 설치 검증, 베타 tag·GitHub Release는 아직 없다. 따라서 현재 품질 문서는 NO-GO이며 `상용급`·`Harper급`·`네이버급` 비교를 사용하지 않는다.
>
> **추가 구현 기록:** `geullint-core`의 `standard` feature에 `StandardPipeline`을 연결했고, 같은 후보·ranker 경계를 native·WASM·CLI(`--engine standard`)에서 호출할 수 있게 했다. 표준 후보는 현재 한 진단 안의 최대 8개 Review 제안으로 묶이며 compact safe fix와 분리된다. 기본 빌드와 all-features 빌드 모두 별도 회귀 테스트를 통과한다.
> **브라우저 검증 기록:** 오프라인 Chromium E2E가 데스크톱·모바일 뷰포트에서 서비스 워커 준비, 별도 원문·교정문, 복사·적용, undo/redo, 사용자 사전 재로드, 네트워크 차단 후 재검사를 검증한다. 적용 직후 undo가 초기 샘플로 되돌아가던 실제 회귀를 수정했고, Pages·release workflow가 잠금된 Playwright Chromium 게이트를 실행한다. Firefox/WebKit과 6개 release runner 검증은 아직 남아 있다.

### Task 1: 기준선과 잠긴 평가 계약

**Files:**
- Modify: `crates/geullint-cli/src/main.rs`
- Modify: `docs/corpus-evaluation.md`
- Create: `crates/geullint-cli/src/evaluation_v2.rs`
- Create: `scripts/check-corpus-leakage.mjs`
- Create: `scripts/evaluate-quality-slices.mjs`
- Create: `corpus/gates/commercial-near-v1.json`
- Create: `docs/accuracy-methodology.md`

- [ ] 문장보다 먼저 출처·문서·작성자 단위로 train/dev/release-holdout을 분할한다.
- [ ] 평가 풀을 자연 문장 20,000개 이상, 인간 수정 edit 5,000개 이상, 정상 문장 10,000개 이상, 8개 이상 장르로 구성한다.
- [ ] 전문가 다중참조 subset과 실제 revision subset을 별도 지표로 보고하고, 주요 장르마다 오류 edit 250개 이상을 확보한다.
- [ ] JSONL v2에 복수 허용 교정, 오류 taxonomy, 장르, 출처 문서 ID, Safe/Review 결과를 기록한다.
- [ ] exact·정규화·자모 5-gram MinHash 누수 검사를 구현하고 중복이 하나라도 있으면 종료 코드 1을 반환한다.
- [ ] 현재 엔진을 공개 benchmark와 release holdout에서 실행해 원시 실패 사례를 바꾸지 않은 baseline 보고서를 만든다.
- [ ] 합성 문장이 quality report 입력으로 들어오면 gate가 실패하도록 provenance 계약을 고정한다.
- [ ] `cargo test -p geullint-cli --all-features`와 `node --test scripts/*corpus*.test.mjs`를 실행해 모두 통과시킨다.
- [ ] Commit: `test: lock independent Korean quality gates`

**Exit gate:** 표본 최소 수와 장르 분포를 충족하고, final 문서 ID와 해시가 엔진 변경 전에 고정되며, train/dev/test 유사 문장 교차가 0건이어야 한다.

### Task 2: 행동 보존 엔진 분리

**Files:**
- Modify: `crates/geullint-core/src/lib.rs`
- Modify: `crates/geullint-core/src/analysis.rs`
- Create: `crates/geullint-core/src/api.rs`
- Create: `crates/geullint-core/src/pipeline.rs`
- Create: `crates/geullint-core/src/analysis/lattice.rs`
- Create: `crates/geullint-core/src/ranking/mod.rs`
- Create: `crates/geullint-core/src/policy.rs`
- Create: `crates/geullint-core/src/planner.rs`
- Create: `crates/geullint-core/src/trace.rs`

- [ ] 기존 공개 API와 직렬화 결과를 golden fixture로 먼저 고정한다.
- [ ] `RuleContext`, `Candidate`, `CandidateScorer`, `DiagnosticArbiter` 경계를 추가한다.
- [ ] 현재 116개 규칙을 새 pipeline 뒤에서 실행하되 결과 순서·범위·수정문을 바꾸지 않는다.
- [ ] 겹침 선택, 수정 후 재검사, 순환 방지 코드를 `planner.rs`로 이동한다.
- [ ] `cargo test --workspace --all-features`를 실행한다.
- [ ] Commit: `refactor: separate analysis and correction pipeline`

**Exit gate:** 기존 Rust 전체 테스트와 세 corpus 출력이 byte-for-byte 동일하고 새 모듈 단위 테스트가 통과해야 한다.

### Task 3: 표준 형태소·어휘 자산

**Files:**
- Create: `data/sources.json`
- Create: `scripts/acquire-training-data.mjs`
- Create: `scripts/build-standard-lexicon.mjs`
- Create: `crates/geullint-core/src/analysis/phonology.rs`
- Create: `crates/geullint-core/src/lexicon/mod.rs`
- Modify: `crates/geullint-core/Cargo.toml`
- Create: `dictionaries/standard-ko-v1.manifest.json`
- Create: `docs/data-card.md`

- [ ] mecab-ko-dic, 허용된 현대 어휘·정규화 자료의 파일별 라이선스와 SHA-256을 allowlist에 고정한다.
- [ ] 계정·약정·불명확 라이선스 출처가 필수 빌드에 들어오면 취득기가 실패하게 한다.
- [ ] 원형, 표면형, 품사, 빈도 구간, 고유명사 성격을 압축 FST/DAWG로 빌드한다.
- [ ] 어간·어미·조사 복수 경로와 종성·ㄹ 예외를 `AnalyzedDocument`에 제공한다.
- [ ] 사용자·workspace·file 사전이 bundled lexicon과 OOV 후보보다 항상 먼저 적용되게 한다.
- [ ] Native와 WASM 분석 결과의 UTF-8 범위·원형·품사 패리티 fixture를 실행한다.
- [ ] `compact`와 `standard` feature 및 자산 manifest를 분리하고, 웹·VS Code·CLI의 일반 사용자 기본값을 `standard`로 고정한다.
- [ ] Commit: `feat: add versioned Korean analysis assets`

**Exit gate:** 모든 자산이 URL·라이선스·해시로 재현되고 standard 어휘 자산 gzip 크기가 20MB 이하여야 한다.

### Task 4: 범용 후보 생성

**Files:**
- Create: `crates/geullint-core/src/candidate/mod.rs`
- Create: `crates/geullint-core/src/candidate/spelling.rs`
- Create: `crates/geullint-core/src/candidate/spacing.rs`
- Create: `crates/geullint-core/src/candidate/grammar.rs`
- Create: `crates/geullint-core/tests/oov_candidates.rs`
- Create: `crates/geullint-core/tests/spacing_generalization.rs`
- Create: `crates/geullint-core/tests/particle_generalization.rs`

- [ ] 초중종성 가중 편집, 두벌식 인접 키, 자모 조합, 삽입·삭제·치환·전도 후보의 실패 테스트를 먼저 작성한다.
- [ ] 활용 가능한 어절과 고유명사를 단순 OOV로 진단하지 않는 hard-negative 테스트를 작성한다.
- [ ] 의존명사와 연결어미, 조사처럼 보이는 어휘 내부 문자열을 구별하는 형태소 경계 테스트를 작성한다.
- [ ] 후보를 edit cost, 형태소 적합도, 빈도, 문맥 n-gram으로 top-32까지 결정론적으로 제한한다.
- [ ] 공개 dev 실오류에서 정답 후보 포함률 top-32 97% 이상을 확인한다.
- [ ] Commit: `feat: generate bounded Korean correction candidates`

**Exit gate:** top-32 후보 recall 97% 이상, 희귀 정상어·고유명사 OOV 경고율 0.1% 이하, Native/WASM 후보 순서 100% 일치.

### Task 5: GeulRank-small 학습과 런타임

**Files:**
- Create: `training/pyproject.toml`
- Create: `training/geullint_training/build_pairs.py`
- Create: `training/geullint_training/train_ranker.py`
- Create: `training/geullint_training/export_onnx.py`
- Create: `training/tests/test_data_split.py`
- Create: `training/tests/test_export_parity.py`
- Create: `models/geulrank-small/manifest.json`
- Create: `models/geulrank-small/MODEL_CARD.md`
- Modify: `crates/geullint-core/src/ranking/mod.rs`
- Modify: `crates/geullint-wasm/src/lib.rs`

- [ ] 실제 인간 수정쌍, 정상 hard negative, train 문서에서만 만든 합성 오류를 서로 구분해 학습 manifest를 생성한다.
- [ ] KoLLA와 release holdout 문서 ID가 학습 manifest에 하나라도 있으면 학습을 중단한다.
- [ ] 6-layer, hidden 256, 10~15M parameter 후보 cross-encoder를 pairwise ranking으로 학습한다.
- [ ] 128-token 문맥에서 top-6~8 후보를 배치 점수화하고 오류 유형 feature를 함께 입력한다.
- [ ] INT8 per-channel ONNX를 내보내고 모델, tokenizer, calibration hash를 manifest에 기록한다.
- [ ] Native와 Web runtime 후보를 같은 fixture로 측정하고, 최종 진단 패리티·크기·지연 기준을 모두 만족하는 backend를 고정한다.
- [ ] `python -m pytest training/tests`와 10,000문장 Native/Web 점수 dead-band 계약을 실행한다.
- [ ] Commit: `feat: rank Korean corrections with a local model`

**Exit gate:** 독립 dev에서 top-1 92%, top-5 98%, ECE 0.05 이하, INT8 모델 18MB 이하.

### Task 6: 안전 정책과 오류 유형 일반화

**Files:**
- Modify: `crates/geullint-core/src/policy.rs`
- Modify: `crates/geullint-core/src/planner.rs`
- Modify: `crates/geullint-core/src/endings.rs`
- Modify: `crates/geullint-core/src/productive.rs`
- Create: `crates/geullint-core/src/style.rs`
- Create: `crates/geullint-core/tests/safe_fix_calibration.rs`
- Create: `crates/geullint-core/tests/style_context.rs`

- [ ] 오류 유형별 top-1 margin, calibrated confidence, 형태소 개선, 금지 문맥을 결합한 정책 테스트를 작성한다.
- [ ] 고유명사·신조어·외래어·인용·숫자·문체 교체는 기본적으로 Review 또는 Abstain으로 고정한다.
- [ ] 조사 양방향, 의존명사, 불규칙 활용, 어미, 문장부호를 표면형 목록이 아닌 후보 family로 이전한다.
- [ ] 문체는 editorial profile과 review-only로 분리하고 복수 허용 제안을 제공한다.
- [ ] gate 미달 family가 자동으로 Safe에서 Review로 강등되는 policy test를 실행한다.
- [ ] Commit: `feat: calibrate safe and review corrections`

**Exit gate:** Safe precision Wilson 95% 하한 99.5%, Review precision 95%, default recall 85%, specificity 99.5%, 범위 밖 수정·순환·비멱등 0건.

### Task 7: 증분 LSP와 공통 제품 API

**Files:**
- Modify: `crates/geullint-lsp/src/lib.rs`
- Modify: `crates/geullint-lsp/src/main.rs`
- Modify: `extensions/vscode-geullint/src/extension.ts`
- Modify: `extensions/vscode-geullint/src/configuration.ts`
- Create: `extensions/vscode-geullint/src/test/incremental-analysis.test.ts`
- Create: `extensions/vscode-geullint/src/test/fix-all.test.ts`

- [ ] LSP `INCREMENTAL` 동기화와 문장·이웃 문장 캐시의 실패 테스트를 작성한다.
- [ ] 변경 범위 밖 진단을 재사용하고 취소·debounce·모델 warm-up을 구현한다.
- [ ] Fix All Safe, preview, 사용자 사전 추가, 이번만 무시, 파일·프로젝트 규칙 끄기를 구현한다.
- [ ] 다중 루트·원격 개발·대형 문서·손상 설정 시나리오를 자동화한다.
- [ ] Commit: `feat: make editor checking incremental and reversible`

**Exit gate:** 증분 한 문장 p95 Native 25ms 이하, full 재검사와 최종 진단 100% 일치, clean VS Code 프로필 E2E 통과.

### Task 8: 상용 검사기 수준의 웹 PWA

**Files:**
- Modify: `apps/playground/index.html`
- Modify: `apps/playground/app.js`
- Modify: `apps/playground/app.css`
- Modify: `apps/playground/worker.js`
- Create: `apps/playground/manifest.webmanifest`
- Create: `apps/playground/storage.js`
- Create: `apps/playground/history.js`
- Create: `scripts/playground-e2e.spec.mjs`

- [ ] 원문·교정문의 동일 수정 범위를 대응 강조하고 진단 선택 시 양쪽 위치로 이동시킨다.
- [ ] 수정별 적용·거절, Fix All Safe, 검토 제안 별도 적용, 다단계 undo/redo를 구현한다.
- [ ] IndexedDB 사용자 사전·설정, 가져오기·내보내기, TXT/Markdown 열기·저장을 구현한다.
- [ ] standard 자산의 설치 진행, SHA-256 검증, 오프라인 준비와 버전을 표시한다.
- [ ] 선택한 최소 문장·진단·버전만 로컬 JSONL로 만드는 피드백 preview와 명시적 GitHub Issue 열기를 구현한다.
- [ ] 키보드 전 구간, 200% 확대, 고대비, reduced motion, 스크린리더 상태 알림을 검증한다.
- [ ] 최초 정적 자산 설치 이후 검사·수정·사전 관리 중 네트워크 요청 0 E2E를 실행한다.
- [ ] Commit: `feat: ship a reversible offline correction workspace`

**Exit gate:** Chromium·Firefox·WebKit, desktop·mobile, offline reload, 파일 입출력, undo/redo, axe 자동 위반 0.

### Task 9: CLI 완성과 저장소 통합

**Files:**
- Modify: `crates/geullint-cli/src/main.rs`
- Create: `crates/geullint-cli/src/commands/check.rs`
- Create: `crates/geullint-cli/src/commands/fix.rs`
- Create: `crates/geullint-cli/src/commands/dictionary.rs`
- Create: `crates/geullint-cli/src/commands/doctor.rs`
- Create: `crates/geullint-cli/src/cache.rs`
- Modify: `crates/geullint-cli/tests/cli.rs`

- [ ] `init`, `doctor`, `check`, `fix --diff`, `--stdin`, `--changed`, `--watch`, dictionary 명령 계약 테스트를 작성한다.
- [ ] `feedback export`와 shell completion을 구현하고 자동 전송이 발생하지 않는 계약 테스트를 작성한다.
- [ ] 수정은 같은 디렉터리 임시 파일과 원자적 교체를 사용하고 BOM·줄바꿈·권한을 보존한다.
- [ ] 내용 hash 캐시와 변경 파일 검사를 구현한다.
- [ ] JSON/SARIF schema version, stdout/stderr, exit code, `NO_COLOR` 계약을 고정한다.
- [ ] pre-commit과 GitHub Action 예제를 실제 fixture 저장소에서 실행한다.
- [ ] Commit: `feat: complete repository-scale checking workflows`

**Exit gate:** 6개 release runner에서 설치, check, diff, atomic fix, cache, JSON/SARIF 호환성 테스트 통과.

### Task 10: 최종 품질·성능·적대 검증

**Files:**
- Modify: `scripts/benchmark-native.mjs`
- Modify: `scripts/benchmark-wasm.mjs`
- Modify: `artifact-budgets.json`
- Create: `scripts/evaluate-commercial-gate.mjs`
- Create: `scripts/red-team-korean.mjs`
- Create: `docs/quality-report-v0.4.0-beta.1.md`

- [ ] 이름·신조어·방언·전문어·혼합 언어·인용·코드·URL·긴 문서 hard negative를 실행한다.
- [ ] UTF-8 경계, 겹친 수정, 손상 자산, 취소, 디스크 오류, OOM 복구, fuzz 테스트를 실행한다.
- [ ] 첫 release holdout을 한 번 실행하고 실패 family만 dev로 재현하되 holdout 문장 자체로 튜닝하지 않는다.
- [ ] 별도 두 번째 holdout에서 같은 gate를 다시 통과시킨다.
- [ ] Native/Web 10,000문장 진단·범위·후보·안전 등급 100% 패리티를 확인한다.
- [ ] standard 웹 추가 다운로드 40MB, Native memory 180MB, Browser memory 220MB, 1,000자 p95 80/200ms와 증분 한 문장 p95 25/60ms 예산을 검사한다.
- [ ] Commit: `test: verify commercial-near quality gates`

**Exit gate:** 설계 문서의 모든 정확도·안전·성능 기준을 서로 다른 holdout 두 개에서 연속 통과.

### Task 11: 공급망, 문서, 베타 릴리스와 홍보 GO

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `.github/workflows/release.yml`
- Modify: `README.md`, `README.en.md`, `README.ja.md`, `README.zh-CN.md`
- Modify: `CHANGELOG.md`, `SECURITY.md`, `THIRD_PARTY_NOTICES.md`
- Create: `docs/model-distribution.md`
- Create: `docs/privacy-threat-model.md`
- Create: `docs/release-go-no-go.md`

- [ ] 모델·사전·엔진을 같은 release manifest, 체크섬, SBOM, attestation으로 검증한다.
- [ ] Windows·macOS·Linux 6개 OS/CPU 아카이브, npm launcher, VSIX, Pages PWA를 clean runner에서 설치한다.
- [ ] 실제 웹·VS Code·CLI 화면과 30초 데모를 릴리스 빌드에서 자동 캡처한다.
- [ ] README 첫 화면에 로컬 실행, 직접 써보기, 독립 품질표, 알려진 한계를 이 순서로 배치한다.
- [ ] 비공식 상용 서비스 비교와 규칙 개수 홍보를 제거하고 재현 가능한 독립 지표만 사용한다.
- [ ] 베타 tag와 GitHub Release를 만들고 Pages·다운로드·attestation·오프라인 PWA를 익명 브라우저에서 재검증한다.
- [ ] 공개 후 재현 가능한 이슈를 회귀 corpus로 편입하고 같은 gate로 patch release를 만든다.
- [ ] Commit: `release: publish independently verified Korean checker beta`

**Exit gate:** 모든 CI·CodeQL·release·Pages·설치·접근성·개인정보·품질 gate 성공 후에만 홍보 문안을 게시한다.

## Codex가 전부 담당하는 범위

- 공개 데이터 검색, 라이선스 allowlist, 자동 취득, 정제, 분할, 누수 검사
- 사전 구축, 모델 학습·양자화, 엔진·웹·VS Code·CLI 구현
- 단위·통합·E2E·접근성·성능·보안·적대 테스트
- README, 다국어 문서, 데이터 카드, 모델 카드, 품질 보고서, 실제 이미지
- 목적별 커밋, PR, GitHub 설정, tag, release, Pages, 릴리스 후 검증

사용자가 제공해야 하는 문장, API 키, 데이터 계정, 라벨, 로컬 명령, 설치 검수, GitHub 업로드 단계는 없다. 계정·본인 인증·약정 서명·유료 인증서가 필요한 제3자 채널은 완성 조건에서 제외한다.
> **Latest implementation record (2026-08-05):** An opt-in deterministic-hash INT8 context ranker is now reproducibly trained from KoLLA annotation pairs, exported as dependency-free JSON plus `MatMulInteger` ONNX, and wired into native `StandardPipeline`, WASM (`evaluate_context`/`lint_context_json`), and CLI (`--engine context`). All learned candidates remain `Review`; the default compact/standard paths and Safe fixes are unchanged. Rebuilding the checked-in model from the derived 3,649-row pair file reproduces JSON, ONNX, and manifest hashes exactly. This does not close the independent human holdout, commercial-near quality, cross-browser, six-runner, or beta-release gates.

> **Latest implementation record (2026-08-05, continued):** The browser playground now ships the optimized `standard` WASM feature, exposes `standard`/`compact`/`context` engine choices, normalizes v2 diagnostics into the existing corrected-sentence UI, and invalidates the Service Worker cache when the engine bundle changes. The VS Code LSP accepts the same `geullint.engine` setting and release packaging builds its server with the `standard` feature; all candidates remain Review-only. The NIKL spelling-correction source is recorded as authorization-only with no placeholder hash, and the acquisition pipeline refuses to treat its request page as data. These changes improve the real user path and provenance boundary but do not close the independent adjudicated holdout or commercial-near quality gate.

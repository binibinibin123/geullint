<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/brand/hero-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="assets/brand/hero-light.svg">
    <img src="assets/brand/hero-light.svg" alt="GeulLint — 글은 로컬에, 교정은 즉시" width="100%">
  </picture>
</p>

<p align="center">
  <a href="README.md"><strong>한국어</strong></a> ·
  <a href="README.en.md">English</a> ·
  <a href="README.ja.md">日本語</a> ·
  <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <a href="https://github.com/binibinibin123/geullint/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/binibinibin123/geullint/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/binibinibin123/geullint/releases"><img alt="Release" src="https://img.shields.io/github/v/release/binibinibin123/geullint?display_name=tag&include_prereleases&sort=semver&color=ff5b35"></a>
  <a href="CHANGELOG.md"><img alt="Early alpha" src="https://img.shields.io/badge/status-early_alpha-dfff38?labelColor=18211c"></a>
  <a href="docs/offline.md"><img alt="Offline first" src="https://img.shields.io/badge/network_requests-0-67ce78?labelColor=18211c"></a>
  <a href="LICENSE"><img alt="MIT License" src="https://img.shields.io/badge/license-MIT-f1efe6?labelColor=18211c"></a>
</p>

<p align="center">
  <strong>글을 밖으로 보내지 않는 오픈소스 한국어 맞춤법 검사기</strong><br>
  맞춤법·띄어쓰기·문법·문체를 브라우저, VS Code, 터미널에서 검사합니다. 문장은 외부 서버로 전송되지 않습니다.
</p>

<p align="center">
  <a href="https://binibinibin123.github.io/geullint/"><strong>지금 문장 검사하기 →</strong></a>
  &nbsp;&nbsp;·&nbsp;&nbsp;
  <a href="#30초-시작">설치하기</a>
  &nbsp;&nbsp;·&nbsp;&nbsp;
  <a href="docs/rules.md">검증된 규칙 보기</a>
</p>

<p align="center">
  <img src="assets/demo/geullint-demo.gif" alt="GeulLint 브라우저 플레이그라운드에서 문장을 검사하고 수정한 뒤 규칙을 검색하는 데모" width="100%">
</p>

## 어디서나 같은 맞춤법 검사

웹에서 문장 하나를 바로 검사하고, 작업이 커지면 같은 검사기를 편집기·터미널·CI로 확장할 수 있습니다.

| 사용 장면 | 할 수 있는 일 |
| --- | --- |
| VS Code에서 쓰기 | 입력 중 실시간 검사하고 한 번의 클릭으로 안전한 교정 적용 |
| CLI로 문서 관리 | 여러 문서와 저장소를 한 명령으로 일괄 검사 |
| CI에서 품질 지키기 | 문서 품질 저하를 자동 차단하고 SARIF 결과 생성 |
| 내 표현에 맞추기 | 사용자 사전, dictionary overlay, 프로젝트 rule pack 추가 |

현재 버전은 **v0.2.0-alpha.1**이며 핵심 규칙 100개를 제공합니다. 문장·진단·텔레메트리는 외부로 전송하지 않습니다.

## 무엇이 다른가

GeulLint는 브라우저·편집기·터미널에서 **완전히 로컬로** 실행되는 한국어 맞춤법·문법 검사기입니다. 어디서 사용하든 같은 Rust 규칙 엔진이 같은 규칙 ID와 교정 결과를 제공합니다.

| | GeulLint |
| --- | --- |
| 개인정보 | 문장·진단·텔레메트리 네트워크 전송 **0회** |
| 공개성 | 규칙 ID, 설명, 예시, 심각도와 테스트를 저장소에서 확인 |
| 자동화 | 사람용 출력, JSON, SARIF 2.1.0, LSP, 종료 코드 |
| 입력 범위 | Markdown 본문, 일반 텍스트, JS/TS·Python·Rust 주석 |
| 확장 | 사용자 사전, dictionary overlay, 프로젝트 rule pack |
| 플랫폼 | Windows·macOS·Linux의 x64/ARM64 |

## 30초 시작

설치 없이 먼저 [웹 플레이그라운드](https://binibinibin123.github.io/geullint/)를 열어보세요. WebAssembly 엔진이 브라우저 안에서만 실행되므로 입력은 어디에도 전송되지 않습니다.

### Windows

아래 설치 스크립트는 최신 Release의 체크섬을 검증하고 사용자 디렉터리에 설치합니다. 실행 전 [원문](install.ps1)을 읽을 수 있습니다.

```powershell
irm https://raw.githubusercontent.com/binibinibin123/geullint/master/install.ps1 | iex
geullint .
```

### macOS · Linux

```bash
curl -fsSL https://raw.githubusercontent.com/binibinibin123/geullint/master/install.sh | sh
geullint .
```

Rust 도구 체인이 이미 있다면 소스에서 고정 설치할 수도 있습니다.

```bash
cargo install --git https://github.com/binibinibin123/geullint --tag v0.2.0-alpha.1 --locked geullint-cli
```

수동 압축 파일은 [GitHub Releases](https://github.com/binibinibin123/geullint/releases)의 대안(fallback)입니다. 사용자가 개별 실행 파일을 찾아 설치하는 방식을 기본 경로로 삼지 않습니다.

| 배포 대상 | 지원 |
| --- | --- |
| Windows x64 | CLI · LSP · VSIX |
| Windows ARM64 | CLI · LSP · VSIX |
| macOS Intel | CLI · LSP · VSIX |
| macOS Apple Silicon | CLI · LSP · VSIX |
| Linux x64 | CLI · LSP · VSIX |
| Linux ARM64 | CLI · LSP · VSIX |

## 실제 출력

```text
$ geullint memo.md
memo.md:1:5: error [spelling.lexical.myeochil] '몇일'은 '며칠'로 쓰는 것이 맞습니다. → 며칠
memo.md:2:4: warning [grammar.conjugation.doe-to-dwae] '되서'는 '돼서'로 쓰는 것이 맞습니다. → 돼서
```

```bash
geullint .                                      # 저장소 검사
geullint --format json docs/                    # JSON
geullint --format sarif docs/ > geullint.sarif  # SARIF 2.1.0
geullint rules --format markdown                # 공개 규칙 목록
geullint --disable spelling.lexical.myeochil note.txt
geullint --dictionary-overlay .geullint.overlay docs/
geullint --rule-pack .geullint-rules.yaml docs/
geullint --corpus corpus/seed-v1.jsonl
geullint --corpus-manifest gold.manifest.json
geullint lsp --stdio
```

CI에서는 설치 후 `geullint .` 한 줄을 실행하면 됩니다. 기본 실패 기준은 `error`이며 `--fail-on warning` 또는 `--fail-on info`로 강화할 수 있습니다. 종료 코드는 `0`(통과), `1`(진단 발견), `2`(설정·실행 오류)입니다.

## 규칙과 품질

GeulLint v0.2.0-alpha.1은 현재 **검수 핵심 규칙 100개**를 제공합니다. 이 숫자는 목표가 아니라 상한이며, 오탐이 확인된 규칙은 수를 유지하려고 대체하지 않고 제거하거나 검토 전용으로 낮춥니다.

| 영역 | 대표 예시 |
| --- | --- |
| 맞춤법·어휘 | `몇일 → 며칠`, `갯수 → 개수`, 혼동 어휘 |
| 문법·활용 | `되서 → 돼서`, 조사·어미·활용 |
| 띄어쓰기 | 검증된 의존 명사와 고정 표현 |
| 기술명 | 선택형 기술 용어 교정 |
| 문체 | 중복·군더더기·편집 제안 |
| 문장부호·타이포그래피 | 공백, 괄호, 기호, 전각 문자 |

새로 검수한 철자 규칙 42개에는 서로 다른 오류 문장 84개와 정상 반례 42개가 있습니다. KoLLA v2의 다중 주석자가 모두 정상으로 판정한 외부 제어 문장 249개에서도 이 알파 카탈로그의 오탐은 0건이었습니다. 표본이 작고 오류 정답 코퍼스가 아니므로 이 결과를 정밀도나 재현율로 홍보하지 않습니다. 재현 정보는 [알파 품질 보고서](docs/quality-report-v0.2.0-alpha.1.md)에 공개합니다.

- [현재 규칙 전체 목록](docs/rules.md)
- [규칙 품질 게이트](docs/quality.md)
- [독립 코퍼스 평가](docs/corpus-evaluation.md)
- [검증 가능한 코퍼스 출처](docs/corpus-sources.md)

## VS Code

<p align="center">
  <img src="assets/screenshots/vscode.png" alt="GeulLint VS Code 확장의 진단, 빠른 수정, 규칙 검색 미리보기" width="100%">
</p>

Release에서 운영체제와 CPU에 맞는 **VSIX**를 받아 VS Code의 `Extensions: Install from VSIX...`로 설치하면 됩니다. VSIX 안에 로컬 언어 서버가 들어 있어 Rust, Node.js, API 키가 필요 없습니다.

- 입력 중 진단과 보수적인 Quick Fix
- 명령 팔레트의 `GeulLint: 규칙 목록 열기`
- `geullint.profile`, `geullint.userDictionary`
- `geullint.dictionaryOverlay`, `geullint.dictionaryOverlayPaths`
- `geullint.rulePacks`

자세한 설정은 [확장 안내](extensions/vscode-geullint/README.md)를 참고하세요.

## 설정과 확장

```json
{
  "profile": "strict",
  "userDictionary": ["GeulLint", "우리제품명"],
  "disabledRules": [
    "spelling.loanword.curated",
    "repetition.adjacent-word"
  ]
}
```

- [프로젝트 rule pack](docs/rule-packs.md): 버전 있는 로컬 YAML 규칙
- [dictionary overlay](docs/dictionary-overlay.md): 팀 고유명사·표면형 사전
- [오프라인·개인정보 경계](docs/offline.md)
- [배포·SBOM·attestation](docs/distribution.md)

## 현재 한계

- 규칙 기반 린터이므로 긴 문맥의 의미 판단이나 창작 문체의 정답을 대신하지 않습니다.
- 안전성이 낮은 제안은 자동 수정하지 않거나 `info` 수준으로 표시합니다.
- matcher contract와 smoke corpus는 엔진 배선 확인용이며 실세계 정확도 수치가 아닙니다.
- 외부 정상 제어군은 249문장뿐이므로 다양한 장르와 작성자를 대표하지 않습니다.
- 외부 독립 코퍼스 평가는 라이선스와 검토 기록을 갖춘 데이터가 있을 때만 수치로 공개합니다.

## 기여

작고 검증 가능한 규칙, 정상 문장을 지키는 반례, 편집기 개선을 환영합니다. 처음이라면 [CONTRIBUTING.md](CONTRIBUTING.md), [규칙 제안 양식](.github/ISSUE_TEMPLATE/rule.yml), [아키텍처](ARCHITECTURE.md), [로드맵](ROADMAP.md)을 먼저 확인하세요.

보안 문제는 공개 이슈 대신 [SECURITY.md](SECURITY.md)의 비공개 신고 절차를 이용해 주세요.

## 라이선스

[MIT](LICENSE) © GeulLint contributors. 내장 형태소 사전과 의존성 고지는 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)에 있습니다.

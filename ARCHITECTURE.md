# GeulLint 아키텍처

GeulLint는 하나의 Rust 규칙 엔진을 여러 표면에서 재사용합니다. CLI, LSP, VS Code, WebAssembly가 별도의 규칙 구현을 갖지 않는 것이 가장 중요한 구조적 원칙입니다.

```text
rules/catalog/*.yaml ─┐
native Rust rules ────┼─> geullint-core ─┬─> geullint-cli
optional morphology ──┘                  ├─> geullint-lsp ─> VS Code
                                         └─> geullint-wasm ─> Web Worker
```

## 구성 요소

- `crates/geullint-core`: 규칙 로딩, 입력 분할, 진단, 안전 수정, 공개 규칙 카탈로그를 담당합니다.
- `crates/geullint-cli`: 파일 탐색, 설정 병합, 사람용·JSON·SARIF 출력, corpus 평가와 종료 코드를 제공합니다.
- `crates/geullint-lsp`: UTF-8 바이트 범위를 편집기용 UTF-16 위치로 변환하고 진단, Code Action, 규칙 카탈로그 메서드를 제공합니다.
- `crates/geullint-wasm`: 같은 코어를 브라우저에 노출합니다. `apps/playground/worker.js`가 메인 UI와 엔진을 분리합니다.
- `extensions/vscode-geullint`: 플랫폼별 LSP 실행 파일을 VSIX 안에 넣고 설정·명령 팔레트·빠른 수정을 연결합니다.
- `rules/`: 사람이 검토 가능한 YAML 카탈로그와 생성 seed입니다.

## 데이터 흐름과 경계

입력 문서는 프로세스 또는 브라우저 탭을 벗어나지 않습니다. core는 네트워크 클라이언트를 의존하지 않으며, playground도 Web Worker에 텍스트만 전달합니다. 사전 overlay와 rule pack은 사용자가 명시한 로컬 파일만 읽습니다.

기본 빌드는 경량 단어 경계 분석기를 사용합니다. Lindera와 `mecab-ko-dic`을 이용한 형태소 분석은 소스 빌드에서 `morphology` feature를 명시했을 때만 포함되며, 기본 릴리스와 WebAssembly 빌드에는 들어가지 않습니다.

Markdown에서는 본문을 검사하고 코드와 링크 목적지를 제외합니다. 지원 프로그래밍 언어에서는 주석만 검사하며 실행 코드와 문자열 리터럴을 건드리지 않습니다. URL·전자우편·파일 경로 같은 식별자는 입력 종류와 관계없이 수정 범위에서 제외합니다. 진단 범위의 기준은 UTF-8 바이트 오프셋이고, 표면별 어댑터가 필요한 좌표 체계로 변환합니다.

## 공개 계약

규칙 ID, 심각도, 메시지, 예시와 안전 수정 여부는 공개 API입니다. `rules/catalog-count.txt`, 생성 규칙 문서, matcher contract와 문장 단위 반례가 드리프트를 막습니다. 카탈로그 수는 생성 결과와 정확히 일치해야 하지만, 규칙 수 자체를 품질이나 출시 목표로 삼지 않습니다. 새 규칙 형식이나 진단 JSON의 호환성을 깨는 변경은 새 버전 계약과 migration 문서가 필요합니다.

배포는 태그에서 여섯 플랫폼 CLI·LSP·VSIX, SHA-256, SPDX SBOM, GitHub attestation을 만듭니다. 자세한 공급망 경계는 [docs/distribution.md](docs/distribution.md)에 있습니다.

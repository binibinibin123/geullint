# GeulLint 배포 가이드

## 사용자 설치 원칙

**npm is optional.** v0.2.0-alpha.1의 기본 경로는 저장소에 공개된 `install.ps1`과 `install.sh`입니다. 설치 스크립트가 운영체제와 CPU를 확인하고 GitHub Release의 네이티브 아카이브와 SHA-256 체크섬을 함께 받은 뒤 사용자 디렉터리에 설치합니다. 사용자는 개별 실행 파일을 직접 고를 필요가 없습니다.

npm 패키지 구조도 함께 빌드·검증하지만, 레지스트리 자격 증명이 설정되기 전에는 README에서 공개된 설치 방법처럼 안내하지 않습니다.

| npm 패키지 | 대상 |
| --- | --- |
| `geullint` | 플랫폼을 선택하는 CLI 런처 |
| `geullint-win32-x64` | Windows x64 |
| `geullint-win32-arm64` | Windows ARM64 |
| `geullint-darwin-x64` | macOS Intel |
| `geullint-darwin-arm64` | macOS Apple Silicon |
| `geullint-linux-x64` | Linux x64 |
| `geullint-linux-arm64` | Linux ARM64 |

GitHub Release의 수동 압축 파일은 자동 설치 스크립트를 실행할 수 없는 격리 환경을 위한 fallback입니다.

## 발행 전 준비

첫 npm 발행 전에 다음 외부 권한이 필요합니다.

1. npm 계정에서 `geullint`와 여섯 플랫폼 패키지를 발행할 권한을 준비합니다.
2. 해당 계정의 자동화 토큰을 GitHub 저장소의 Actions secret `NPM_TOKEN`으로 추가합니다.
3. GitHub Actions가 기본 `GITHUB_TOKEN`으로 Release를 만들 수 있도록 저장소 Actions 권한을 `Read and write permissions`로 둡니다.

이 권한이 없어도 GitHub Release와 모든 플랫폼 아카이브·VSIX는 정상적으로 생성됩니다. `NPM_TOKEN`이 있을 때만 별도의 선택 단계가 npm 플랫폼 패키지와 런처를 발행합니다. 소스 저장소가 공개되어 있다는 사실만으로 npm 레지스트리에 패키지를 올릴 수는 없습니다.

## 버전과 태그

Rust 워크스페이스와 여섯 npm 플랫폼 패키지, 런처 `package.json`의 버전은 같아야 합니다. 현재 알파 버전은 모두 `0.2.0-alpha.1`이고, 발행 태그는 `v0.2.0-alpha.1`입니다.

```bash
git tag v0.2.0-alpha.1
git push origin v0.2.0-alpha.1
```

`.github/workflows/release.yml`은 태그 버전과 `geullint-cli`의 Cargo 버전, 모든 npm 패키지 버전이 일치하는지 먼저 검증합니다. 이후 각 운영체제에서 CLI를 빌드하고 다음 순서로 진행합니다.

1. 여섯 플랫폼에서 네이티브 CLI와 아카이브를 빌드합니다.
2. Windows는 ZIP, macOS·Linux는 `tar.gz` 아카이브와 SHA-256 파일을 GitHub Release에 첨부합니다.
3. 각 플랫폼에서 언어 서버를 빌드해 **여섯 플랫폼별 VSIX**에 포함하고 GitHub Release에 첨부합니다. VSIX 사용자는 Rust·npm·외부 API를 설치할 필요가 없습니다.
4. `NPM_TOKEN`이 설정된 경우에만 여섯 플랫폼 패키지와 `geullint` 런처를 npm에 발행합니다. 이 선택 단계의 유무는 GitHub Release 생성에 영향을 주지 않습니다.

재시도 전에 npm에 일부 플랫폼 패키지만 이미 발행되었는지 확인하세요. npm 패키지 버전은 덮어쓸 수 없습니다. 새 버전을 올리는 것이 정상 복구 경로입니다.

## 공급망 증거와 사전 고지

릴리스 워크플로는 각 네이티브 바이너리와 VSIX에 SPDX **SBOM**을 생성하고, GitHub artifact **attestation**으로 빌드 출처와 SBOM을 서명합니다. VSIX는 다운로드한 파일을 바로 검증할 수 있습니다.

```bash
gh attestation verify geullint-v0.2.0-alpha.1-vscode-win32-x64.vsix \
  --repo binibinibin123/geullint \
  --predicate-type https://spdx.dev/Document/v2.3 \
  --signer-workflow binibinibin123/geullint/.github/workflows/release.yml
```

CLI의 서명 대상은 압축 파일이 아니라 그 안의 `geullint` 또는 `geullint.exe`입니다. 먼저 함께 배포된 `.sha256`으로 압축 파일을 확인하고 압축을 푼 뒤, 같은 명령의 첫 번째 인수에 실행 파일 경로를 지정합니다.

내장 형태소 분석에는 Lindera와 `mecab-ko-dic` 데이터를 사용합니다. 데이터 출처·라이선스·향후 별도 사전 추가 원칙은 [`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md)에, Apache-2.0 전문은 [`LICENSES/Apache-2.0.txt`](../LICENSES/Apache-2.0.txt)에 포함합니다. 이 고지는 npm 플랫폼 패키지, GitHub 아카이브, VSIX에 함께 넣습니다.

## 로컬 검증

다음은 레지스트리에 올리지 않고 실행할 수 있는 배포 검증입니다.

```powershell
node --test packages/npm/geullint/test/geullint.test.js
node --test scripts/release-workflow.test.mjs
node --test scripts/release-smoke.test.mjs
node --test scripts/readme-contract.test.mjs

Push-Location packages/npm/geullint
npm pack --dry-run --json
Pop-Location

cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

플랫폼 패키지에 실제 실행 파일이 들어가는 것은 릴리스 워크플로의 빌드 단계입니다. 따라서 Git에 바이너리를 커밋하거나 사용자가 `.exe` 파일을 직접 설치할 필요가 없습니다.

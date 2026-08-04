# GeulLint 배포 가이드

## 사용자 설치 원칙

**npm is optional.** v0.4.0-beta.1의 기본 경로는 저장소에 공개된 `install.ps1`과 `install.sh`입니다. 설치 스크립트가 운영체제와 CPU를 확인하고 GitHub Release의 네이티브 아카이브와 SHA-256 체크섬을 함께 받은 뒤 사용자 디렉터리에 설치합니다. 사용자는 개별 실행 파일을 직접 고를 필요가 없습니다.

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

GitHub Release와 선택적 npm 발행 전에 다음 권한을 확인합니다.

1. npm으로도 발행할 때만, npm 계정에서 `geullint`와 여섯 플랫폼 패키지를 발행할 권한을 준비합니다.
2. npm으로도 발행할 때만, 해당 계정의 자동화 토큰을 GitHub 저장소의 Actions secret `NPM_TOKEN`으로 추가합니다.
3. 저장소나 조직의 Actions 정책이 릴리스 워크플로에 명시된 `contents: write`, `id-token: write`, `attestations: write` 권한 요청을 허용하는지 확인합니다. 저장소의 기본 워크플로 권한은 읽기 전용으로 유지할 수 있습니다.

npm 발행 권한과 `NPM_TOKEN`이 없어도 GitHub Release와 모든 플랫폼 아카이브·VSIX는 정상적으로 생성됩니다. 단, 3번의 GitHub Actions 권한 요청은 허용되어야 합니다. `NPM_TOKEN`이 있을 때만 별도의 선택 단계가 npm 플랫폼 패키지와 런처를 발행합니다. 소스 저장소가 공개되어 있다는 사실만으로 npm 레지스트리에 패키지를 올릴 수는 없습니다.

## 버전과 태그

Rust 워크스페이스와 여섯 npm 플랫폼 패키지, 런처 `package.json`의 버전은 같아야 합니다. 현재 베타 버전은 모두 `0.4.0-beta.1`이고, 발행 태그는 `v0.4.0-beta.1`입니다.

```bash
git tag v0.4.0-beta.1
git push origin v0.4.0-beta.1
```

`.github/workflows/release.yml`은 태그 버전과 `geullint-cli`의 Cargo 버전, 모든 npm 패키지 버전이 일치하는지 먼저 검증합니다. 이후 각 운영체제에서 CLI를 빌드하고 다음 순서로 진행합니다.

1. 여섯 플랫폼에서 네이티브 CLI와 아카이브를 빌드합니다.
2. Windows는 ZIP, macOS·Linux는 `tar.gz` 아카이브와 SHA-256 파일을 GitHub Release에 첨부합니다.
3. 각 플랫폼에서 언어 서버를 빌드해 **여섯 플랫폼별 VSIX**에 포함하고 GitHub Release에 첨부합니다. VSIX 사용자는 Rust·npm·외부 API를 설치할 필요가 없습니다.
4. `NPM_TOKEN`이 설정된 경우에만 여섯 플랫폼 패키지와 `geullint` 런처를 npm에 발행합니다. 이 선택 단계의 유무는 GitHub Release 생성에 영향을 주지 않습니다.

재시도 전에 npm에 일부 플랫폼 패키지만 이미 발행되었는지 확인하세요. npm 패키지 버전은 덮어쓸 수 없습니다. 새 버전을 올리는 것이 정상 복구 경로입니다.

## 공급망 증거와 사전 고지

릴리스 워크플로는 사용자가 다운로드하는 각 네이티브 아카이브와 VSIX에 SPDX **SBOM**을 생성하고, 그 아카이브·VSIX 파일을 GitHub artifact **attestation**의 대상으로 서명합니다. 설치 스크립트도 별도로 빌드 출처를 증명합니다.

```bash
gh attestation verify geullint-v0.4.0-beta.1-vscode-win32-x64.vsix \
  --repo binibinibin123/geullint \
  --predicate-type https://spdx.dev/Document/v2.3 \
  --signer-workflow binibinibin123/geullint/.github/workflows/release.yml
```

CLI의 attestation 대상은 내부의 `geullint` 또는 `geullint.exe`가 아니라 다운로드한 `.zip` 또는 `.tar.gz` 아카이브 자체입니다. 함께 배포된 `.sha256`으로 파일 무결성을 확인하고, `gh attestation verify`의 첫 번째 인수에도 그 아카이브 경로를 지정합니다. 압축 내부 실행 파일은 별도로 attestation되지 않습니다.

소스 빌드의 선택적 `morphology` 기능은 Lindera와 `mecab-ko-dic` 데이터를 사용하지만 기본 릴리스에는 포함하지 않습니다. 데이터 출처·라이선스·향후 별도 사전 추가 원칙은 [`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md)에, Apache-2.0 전문은 [`LICENSES/Apache-2.0.txt`](../LICENSES/Apache-2.0.txt)에 기록합니다. 이 고지는 npm 플랫폼 패키지, GitHub 아카이브, VSIX에 함께 넣습니다.

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

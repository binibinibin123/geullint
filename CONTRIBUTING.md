# GeulLint 기여 가이드

GeulLint는 한국어 글을 외부로 보내지 않고 점검할 수 있어야 한다는 원칙에서 출발했습니다. 새로운 규칙, 정상 문장을 지키는 반례, 문서와 편집기 개선을 모두 환영합니다.

## 시작하기

Rust 1.96.0과 Node.js 22 이상을 준비한 뒤 저장소를 포크하고 브랜치를 만드세요.

```bash
cargo test --workspace
node --test scripts/*.test.mjs

cd extensions/vscode-geullint
npm ci
npm test
```

변경을 제출하기 전에는 다음 품질 검사를 실행합니다.

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## 규칙을 추가할 때

고정 치환 규칙은 `rules/catalog/*.yaml`과 생성 스크립트로, 문맥 판단이 필요한 규칙은 테스트가 있는 Rust 코드로 관리합니다. 규칙 ID는 `영역.하위영역.이름`처럼 안정적인 네임스페이스를 사용합니다.

새 공개 규칙에는 최소한 다음 사례가 필요합니다.

- 실제 오류를 잡는 양성 사례
- 비슷하지만 올바른 정상 사례
- 고유명사·인용·코드 등 예외 사례
- UTF-8 바이트 범위와 교체 결과
- 안전 수정의 멱등성
- 다른 규칙과 중복 진단이 생기지 않는 조합 사례

`safeFix: true`는 해당 범위의 치환이 문맥과 무관하게 안전할 때만 허용됩니다. 판단이 필요한 제안은 심각도를 낮추고 자동 수정하지 마세요. 상세 기준은 [규칙 품질 게이트](docs/quality.md)를 따릅니다.

## Pull Request

한 PR은 한 가지 목적에 집중해 주세요. 본문에는 문제, 접근 방법, 검증 명령과 결과, 사용자에게 보이는 변경을 적습니다. 공개 API·규칙 ID·출력 형식을 바꾼다면 호환성 영향도 설명합니다.

모든 진단은 결정적이고 로컬이어야 합니다. 새 네트워크 요청, 텔레메트리, 실행 코드 또는 문자열 리터럴 검사는 프로젝트 원칙과 충돌하므로 받지 않습니다.

기여하면 제출한 코드와 문서가 저장소의 [MIT 라이선스](LICENSE)로 배포되는 데 동의하게 됩니다. 서로를 존중하는 협업 기준은 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)를 따릅니다.

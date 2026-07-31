# 보안 정책

GeulLint는 입력 문서를 로컬에서만 처리합니다. 그럼에도 설치기, Release 아카이브, VS Code 확장, rule pack 파서에서 공급망 또는 임의 코드 실행 문제가 발견될 수 있습니다.

## 비공개 신고

취약점은 공개 Issue로 올리지 말고 GitHub의 [비공개 보안 권고](https://github.com/binibinibin123/geullint/security/advisories/new)를 사용해 주세요. 재현 절차, 영향을 받는 버전과 플랫폼, 예상 영향, 가능한 완화책을 포함하면 확인에 도움이 됩니다.

프로젝트는 신고 접수 후 가능한 한 빠르게 수신을 확인하고, 재현 여부와 공개 일정을 신고자와 조율합니다. 수정 전에는 취약점 세부 정보를 공개하지 말아 주세요. 유효한 신고자는 원한다면 릴리스 고지에 이름을 올릴 수 있습니다.

## 지원 범위

| 버전 | 보안 수정 |
| --- | --- |
| 최신 공개 Release | 지원 |
| 이전 Release | 최신 버전에서 재현되는 경우 우선 검토 |
| 임의 커밋·비공식 빌드 | 최선의 노력 |

## 릴리스 검증

공식 Release는 SHA-256 체크섬, SPDX SBOM, GitHub artifact attestation을 제공합니다. 설치 스크립트는 아카이브를 풀기 전에 체크섬을 확인합니다.

```bash
gh attestation verify PATH/TO/ARTIFACT -R binibinibin123/geullint
```

GeulLint 코어와 playground는 진단을 위해 네트워크 요청을 하지 않습니다. 이 경계가 깨지는 동작, 텍스트 유출, 경로 탈출, 안전 수정으로 인한 의도하지 않은 파일 변경, 악성 rule pack 처리 문제는 보안 이슈로 간주합니다.

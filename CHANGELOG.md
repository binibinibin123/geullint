# Changelog

이 파일은 GeulLint의 사용자에게 보이는 변경을 기록합니다. 형식은 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)를 참고하며 버전은 Semantic Versioning을 따릅니다.

## [Unreleased]

## [0.2.0-alpha.1] - 2026-07-31

첫 공개 알파 버전입니다.

### Added

- 맞춤법·띄어쓰기·문법·문체를 검사하는 100개 핵심 규칙
- Markdown·일반 텍스트와 JavaScript·TypeScript·Python·Rust 주석용 CLI
- 사람용, JSON, SARIF 2.1.0 출력과 CI용 종료 코드
- 사용자 사전, dictionary overlay, 버전 있는 YAML rule pack
- 진단, Quick Fix, 검색 가능한 규칙 카탈로그를 제공하는 LSP와 VS Code 확장
- 서버 요청 없이 실행되는 WebAssembly 플레이그라운드와 4개 언어 UI
- Windows·macOS·Linux x64/ARM64용 체크섬 Release, VSIX, SBOM과 build attestation
- Release를 자동 선택하고 SHA-256을 확인하는 PowerShell·POSIX 설치 스크립트
- 새 철자 규칙 42개를 위한 오류 문장 84개와 정상 반례 42개
- KoLLA v2 정상 제어 문장 249개의 오탐 검사와 재현 정보

### Security

- 문장, 진단 결과, 텔레메트리를 전송하지 않는 오프라인 경계를 테스트로 고정
- 외부 corpus를 평가하기 전 라이선스, 출처, SHA-256, 독립 검토 기록을 검증

[Unreleased]: https://github.com/binibinibin123/geullint/compare/v0.2.0-alpha.1...HEAD
[0.2.0-alpha.1]: https://github.com/binibinibin123/geullint/releases/tag/v0.2.0-alpha.1

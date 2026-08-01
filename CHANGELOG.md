# Changelog

이 파일은 GeulLint의 사용자에게 보이는 변경을 기록합니다. 형식은 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)를 참고하며 버전은 Semantic Versioning을 따릅니다.

## [Unreleased]

## [0.3.0-alpha.1] - 2026-08-02

### Added

- `-읍니다`, `-십시요`, `아니예요`, `않되-`와 의존 명사 일곱 계열을 검사하는 11개 문맥 규칙
- 서로 다른 144개 문장으로 구성한 출시 안전 회귀와 manifest 검증 게이트
- 1 KiB·100 KiB·1 MiB 일반 텍스트, Markdown, TypeScript 성능 fixture와 재현 가능한 Native·WASM 측정 도구
- 실제 릴리스 아카이브·VSIX의 SPDX SBOM과 GitHub build attestation
- 디렉터리 검사 시 `.gitignore`와 `.geullintignore`를 따르는 텍스트 파일 탐색

### Changed

- 규칙별 반복 검색을 단일 패스 경계 인식 matcher로 교체
- Native와 WebAssembly가 동일한 경량 소스 범위 분석기를 사용하도록 통합
- JavaScript·TypeScript 정규식과 Markdown 가변 백틱 코드 구역을 보수적으로 제외
- 웹 검사 화면이 안전 수정과 검토 제안을 반영한 교정문을 함께 표시
- 연쇄된 교정도 한 번의 `--fix` 또는 웹 요청에서 안정 상태까지 적용
- 공개 규칙 카탈로그의 임시 영문 제목을 검수한 한국어 제목·설명으로 교체
- 설치 예시를 감사한 태그와 버전에 고정하고, 태그 커밋 검증 뒤에만 릴리스하도록 강화

### Fixed

- 조사·어휘 규칙이 긴 단어 안쪽을 잘못 고치거나 문체 선택을 자동 수정하던 문제
- `은은하다` 같은 정상어를 중복 조사로 오인해 안전 수정하던 문제
- 소스 문자열·정규식, Markdown 코드·링크 주소와 URL·이메일·파일명·해시태그의 교정 범위를 잘못 계산하던 문제
- 연속 쉼표와 문장부호 앞 공백이 겹치는 진단과 교정을 만들던 문제
- VS Code 확장에 Language Client 런타임과 해당 제3자 라이선스 고지가 빠지던 문제
- artifact 전송 뒤 macOS·Linux npm 실행 파일 권한과 MIT 라이선스가 빠지던 문제

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

[Unreleased]: https://github.com/binibinibin123/geullint/compare/v0.3.0-alpha.1...HEAD
[0.3.0-alpha.1]: https://github.com/binibinibin123/geullint/releases/tag/v0.3.0-alpha.1
[0.2.0-alpha.1]: https://github.com/binibinibin123/geullint/releases/tag/v0.2.0-alpha.1

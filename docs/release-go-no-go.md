# 베타 릴리스 GO / NO-GO

## GO 조건

- `cargo fmt --check`, Clippy 전체, workspace all-features 테스트 통과
- Native·WASM 진단·범위·수정문 패리티 통과
- 독립 자연 문장·인간 수정·정상 문장·8개 장르·두 holdout을 `commercial-near-v1` 게이트가 승인
- Safe precision 하한과 specificity, 성능·파일 크기 예산 통과
- red-team, 접근성, offline reload, CLI/LSP/VSIX smoke 통과
- 릴리스 아카이브·VSIX의 체크섬, SBOM, attestation과 설치 문서 일치

## NO-GO 조건

- 합성 문장만으로 정밀도·재현율을 홍보해야 하는 경우
- holdout 문장을 튜닝 자료로 되돌린 흔적이 있는 경우
- 원문을 외부로 보내는 경로, API 키, 계정 로그인이 필수인 경우
- Safe 제안의 오탐, 범위 밖 수정, 비멱등 수정, 손상 자산 복구 실패
- 위 항목 중 하나라도 측정되지 않았거나 재현 명령이 없는 경우

현재 저장소는 독립 코퍼스 규모와 두 holdout 조건을 아직 충족하지 않았으므로 품질 GO가 아니라 **NO-GO / 개발 계속** 상태다. 작은 회귀 코퍼스 통과를 상용 동급 인증으로 표현하지 않는다.

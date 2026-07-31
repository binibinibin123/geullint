# GeulLint 로드맵

이 문서는 다음 버전에서 다룰 작업을 정리합니다. 순서와 범위는 기여와 검증 결과에 따라 달라질 수 있습니다.

## v0.2 alpha

- [x] 100개 핵심 규칙
- [x] 새 철자 규칙별 오류 문장과 정상 반례
- [x] KoLLA v2 정상 제어군 오탐 감사
- [x] CLI의 사람용·JSON·SARIF 출력과 corpus 평가
- [x] LSP, VS Code Quick Fix, 검색 가능한 규칙 목록
- [x] 완전 로컬 WebAssembly 플레이그라운드와 4개 언어 UI
- [x] 6개 OS/CPU Release, 체크섬, VSIX, SBOM, attestation
- [x] 원라인 설치기와 다국어 저장소 문서

## 다음 단계 — 독립 검증과 생태계

- [ ] 라이선스·SHA-256·이중 검토 기록이 있는 독립 gold corpus 공개
- [ ] 영역별 precision/recall과 Wilson 구간을 릴리스별로 기록
- [ ] 오탐 회귀 제출을 위한 최소 재현 corpus 형식 안정화
- [ ] npm trusted publishing 또는 검증 가능한 대체 패키지 채널
- [ ] Homebrew·WinGet 등 커뮤니티 패키지 매니저 제안
- [ ] 규칙 카탈로그의 언어별 설명 번역

## 장기 방향

- 형태소 분석을 활용하되 불확실한 문맥에서는 진단을 억제하는 보수적 규칙 확대
- 대규모 저장소에서 변경 파일만 점검하는 캐시와 증분 분석
- 다른 편집기에서 재사용 가능한 LSP 설치 문서와 패키지
- 외부 서비스 없이 팀 사전과 조직 rule pack을 안전하게 공유하는 버전 계약

## 하지 않는 것

문장을 서버로 보내야만 사용할 수 있는 기능, 사용자 추적, 내용을 확인할 수 없는 비공개 규칙은 계획하지 않습니다.

제안은 [Feature request](https://github.com/binibinibin123/geullint/issues/new?template=feature.yml), 규칙은 [Rule proposal](https://github.com/binibinibin123/geullint/issues/new?template=rule.yml) 양식을 사용해 주세요.

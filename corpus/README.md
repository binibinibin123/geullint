# 평가 코퍼스 디렉터리

`seed-v1.jsonl`은 모든 안정 규칙 ID의 실행 여부를 확인하는 소형 smoke corpus입니다. 통계적 품질을 주장하는 gold corpus가 아닙니다.

`curated-alpha-v1.jsonl`은 새로 검수한 철자 규칙의 서로 다른 오류 문장과 정상 반례를 담은 프로젝트 내부 문장 회귀 자료입니다. 이 자료도 독립 gold corpus로 부르지 않습니다.

`safety-regressions-v1.jsonl`은 프로젝트가 직접 작성한 적대적 회귀 모음입니다. 오류 72건과 정상 반례 72건을 업무·뉴스·교육·공공·대화·서사·기술·개발자 문맥에 각각 18건씩 배치하고, 44개 규칙 ID를 검사합니다. 이는 실제 사용자 분포를 표본 추출한 독립 gold corpus가 아니며 일반적인 정밀도·재현율의 근거로 사용하지 않습니다.

`safety-regressions-v1.policy.json`은 사례 수, 장르·입력 종류·프로필 분포, 중복, 문장 유사도, 필수 규칙 ID를 구조적으로 고정합니다. `safety-regressions-v1.manifest.json`은 MIT 라이선스, 저장소 출처와 코퍼스 원문의 SHA-256을 기록합니다. 각 오류 주석의 유일한 `original`에서 UTF-8 바이트 범위를 결정하고 제안·규칙 ID를 대조하며, `expectedFixedText`는 안전 수정 결과 또는 검토 전 원문을 정확히 고정합니다.

외부 코퍼스 원문은 이 저장소에 추가하지 않습니다. 취득 권한, 라이선스, 원 출처, SHA-256을 기록한 뒤 로컬에서 [`docs/corpus-evaluation.md`](../docs/corpus-evaluation.md)의 manifest 방식으로 평가하세요.

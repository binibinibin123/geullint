# 검수 규칙 품질 기준

GeulLint는 규칙 수를 출시 목표로 삼지 않습니다. 현재 알파 카탈로그의 상한은 100개이며 오탐이 발견된 규칙은 즉시 삭제하거나 `review`로 낮춥니다.

## 서로 다른 세 종류의 테스트

1. **matcher contract**: 규칙 로딩, Markdown·코드 주석 범위, UTF-8 위치, 수정 멱등성을 검사합니다.
2. **문장 단위 회귀**: 실제 문장 형태의 오류와 정상 반례를 검사합니다. [`curated-alpha-v1.jsonl`](../corpus/curated-alpha-v1.jsonl)이 여기에 해당합니다.
3. **외부 corpus 평가**: 저장소와 독립된 자료의 precision·recall·specificity를 측정합니다. 라이선스·원본 URL·SHA-256을 manifest로 고정합니다.

matcher contract와 smoke corpus는 언어 정확도 사례 수로 홍보하지 않습니다.

## 자동 게이트

- `rules/catalog-count.txt`는 실제 카탈로그와 일치하고 100을 넘지 않아야 합니다.
- 공개 규칙 ID, source matcher와 문서 anchor는 중복될 수 없습니다.
- 문맥 없는 복합명사 띄어쓰기와 제품명 대소문자 규칙은 기본 카탈로그에 넣지 않습니다.
- 모든 제안은 같은 규칙으로 다시 진단되지 않아야 합니다.
- `safeFix: true`는 수정 뒤 진단 제거와 두 번째 수정의 멱등성을 통과해야 합니다.
- 정상 반례에서 한 번이라도 오탐이 재현되면 `review`로 낮추거나 제거합니다.

## 공개 수치

알파별 측정 결과와 원본 해시는 [v0.2.0-alpha.1 품질 보고서](quality-report-v0.2.0-alpha.1.md)에 기록합니다. 오류가 포함된 독립 gold corpus를 두 명 이상이 검토하기 전에는 외부 precision·recall을 주장하지 않습니다.

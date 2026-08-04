# 검수 규칙 품질 기준

GeulLint는 규칙 수를 출시 목표로 삼지 않습니다. 오탐이 발견된 규칙은 즉시 삭제하거나 `review`로 낮춥니다.

## 서로 다른 세 종류의 테스트

1. **분석 경로 계약**: Markdown·코드 주석 범위, 한국어 단어 경계, UTF-8 위치, 수정 멱등성을 검사합니다. 기본 네이티브·브라우저 빌드는 같은 경량 단어 경계 규칙으로 동작합니다. 소스 빌드에서 `morphology` feature를 명시하면 로컬 형태소 사전으로 품사 정보를 보강합니다.
2. **문장 단위 회귀**: 실제 문장 형태의 오류와 정상 반례를 검사합니다. 소규모 규칙 사례는 [`curated-alpha-v1.jsonl`](../corpus/curated-alpha-v1.jsonl), 출시 안전 회귀는 [`safety-regressions-v1.jsonl`](../corpus/safety-regressions-v1.jsonl)이 담당합니다.
3. **외부 corpus 평가**: 저장소와 독립된 자료의 precision·recall·specificity를 측정합니다. 라이선스·원본 URL·SHA-256을 manifest로 고정합니다.

matcher contract와 smoke corpus는 언어 정확도 사례 수로 홍보하지 않습니다.

## 자동 게이트

- `rules/catalog-count.txt`는 실제 카탈로그와 일치해야 합니다. 이 수는 품질 점수가 아닙니다.
- 공개 규칙 ID, source matcher와 문서 anchor는 중복될 수 없습니다.
- 문맥 없는 복합명사 띄어쓰기와 제품명 대소문자 규칙은 기본 카탈로그에 넣지 않습니다.
- 모든 제안은 같은 규칙으로 다시 진단되지 않아야 합니다.
- `safeFix: true`는 수정 뒤 진단 제거와 두 번째 수정의 멱등성을 통과해야 합니다.
- 말투·문체에 따라 의도가 달라질 수 있는 제안은 `safeFix: false`여야 합니다. 예를 들어 `감사해용 → 감사해요`는 검토 제안입니다.
- 정상 반례에서 한 번이라도 오탐이 재현되면 `review`로 낮추거나 제거합니다.
- 출시 전에는 프로젝트 소유 안전 회귀 144건의 구조 정책과 manifest SHA-256을 확인한 뒤, 규칙 ID·UTF-8 바이트 범위·제안·수정 결과를 실제 CLI 출력과 대조합니다.

안전 회귀 모음은 오류 72건과 정상 72건, 8개 장르, 44개 규칙 ID로 구성됩니다. 중복과 지나치게 비슷한 문장을 막고 코드 문자열과 주석을 함께 다루지만, 프로젝트가 직접 작성한 적대적 테스트이므로 독립 gold corpus 또는 전체 언어 성능의 증거가 아닙니다.

## 공개 수치

베타 측정 결과와 원본 해시는 [v0.4.0-beta.1 품질 보고서](quality-report-v0.4.0-beta.1.md)에 기록합니다. 이전 기준선은 [v0.3.0-alpha.1 보고서](quality-report-v0.3.0-alpha.1.md)에 보존합니다. 오류가 포함된 독립 gold corpus를 두 명 이상이 검토하기 전에는 외부 precision·recall을 주장하지 않습니다.

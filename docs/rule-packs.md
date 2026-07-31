# 오프라인 rule pack DSL

GeulLint의 기본 규칙은 번들 YAML로 제공되며, 프로젝트·조직 고유의 고신뢰 규칙은 별도 **rule pack**으로 추가할 수 있습니다. pack은 로컬 UTF-8 YAML 파일만 읽습니다. 검사·자동수정·corpus 평가는 네트워크, API 키, 계정을 사용하지 않습니다.

## v1 형식

```yaml
version: 1
language: ko
rules:
  - id: spelling.project.product-name
    severity: warning
    message: "프로젝트 제품명 표기를 확인하세요."
    safeFix: true
    replacements:
      - from: 글린트
        to: GeulLint
```

번들 예제는 [`examples/rule-pack-v1.yaml`](../examples/rule-pack-v1.yaml)에 있습니다.

| 필드 | 설명 |
| --- | --- |
| `version` | 현재 반드시 숫자 `1` |
| `language` | 현재 반드시 `ko` |
| `rules[].id` | 고유하고 안정적인 이름공간 ID. 번들 규칙 또는 다른 pack과 겹치면 거부됨 |
| `severity` | `error`, `warning`, `info` |
| `profile` | 선택. `default`(기본), `strict`, `editorial` |
| `message` | 사용자에게 보이는 설명 |
| `safeFix` | `true`이면 정확한 치환만 `--fix`로 자동 적용할 수 있음 |
| `replacements` | 하나 이상의 `{from, to}`. 빈 값과 같은 규칙 안의 중복 `from`은 거부됨 |

## 사용

```bash
geullint --rule-pack .geullint-rules.yaml docs/
geullint --fix --rule-pack .geullint-rules.yaml docs/memo.md
```

여러 pack을 지정할 수도 있습니다. 모든 ID는 번들 규칙과 다른 pack 사이에서도 고유해야 합니다.

```bash
geullint \
  --rule-pack terminology-rules.yaml \
  --rule-pack editorial-rules.yaml \
  docs/
```

새 pack도 gold corpus로 검증합니다. `--rule-pack`은 `--corpus`와 함께 쓸 수 있으므로, corpus에 새 rule ID를 기대값으로 기록해 오탐·누락을 CI에서 실패시킬 수 있습니다.

```bash
geullint \
  --rule-pack terminology-rules.yaml \
  --corpus corpus/project-rules.jsonl
```

잘못된 YAML·지원하지 않는 버전·빈 규칙·빈 치환·ID 충돌은 종료 코드 `2`로 거부합니다. pack은 CLI와 corpus 평가에 적용됩니다.

## VS Code

VS Code 확장은 `geullint.rulePacks`로 같은 로컬 YAML을 LSP 진단과 안전 Quick Fix에 적용합니다.

```json
{
  "geullint.rulePacks": [
    ".geullint-rules.yaml",
    "rules/editorial.yaml"
  ]
}
```

상대 경로는 첫 워크스페이스 폴더를 기준으로 해석합니다. 다중 루트 워크스페이스에서는 혼동을 피하기 위해 절대 경로를 사용하세요. 확장은 지정한 로컬 파일만 읽으며 네트워크·API 키·계정을 사용하지 않습니다.

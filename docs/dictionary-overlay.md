# 오프라인 사전 overlay

GeulLint의 네이티브 CLI·LSP 기본 형태소 사전은 바이너리에 포함되어 있습니다. 프로젝트 고유명사·제품명·도메인 용어는 별도의 **overlay** 파일로 더할 수 있습니다. 검사 중에는 파일 시스템에서 지정한 파일만 읽고, 네트워크·API·계정·텔레메트리를 사용하지 않습니다.

## 내장 사전 snapshot

현재 배포에 포함되는 형태소 사전은 `mecab-ko-dic-2.1.1-20180720`을 Lindera `lindera-ko-dic` 4.0.1로 패키징한 것입니다. [`dictionaries/embedded-mecab-ko-dic-v1.json`](../dictionaries/embedded-mecab-ko-dic-v1.json)은 crate SHA-256, 원본 버전·URL·라이선스를 고정합니다. CI는 이 값이 `Cargo.lock`과 [`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md)에 적힌 고지와 계속 일치하는지 검사합니다.

이 snapshot은 런타임 다운로드가 아닙니다. 별도 국립국어원 사전 데이터를 추가하려면 해당 데이터의 권한·원본 hash·출처·라이선스를 별도로 검증한 새 snapshot manifest를 만들어야 하며, API 키를 사용자나 검사 런타임에 요구하지 않습니다.

## 파일 형식

UTF-8 텍스트 파일의 첫 줄은 반드시 `geullint-overlay-v1`입니다. 다음 줄부터는 표면형과 품사를 탭으로 구분합니다.

```text
geullint-overlay-v1
GeulLint	NNP
프로젝트명	NNP
```

각 줄의 `표면형<TAB>POS` 중 POS는 `NNP`, `NNG`, `VV`처럼 형태소 태그를 기록합니다. CLI 린터는 overlay 표면형을 사전 인식 어휘 규칙의 예외로 사용합니다. Rust의 `MorphAnalyzer::with_overlay`는 같은 표면형이 형태소 분석 결과에 나오면 overlay POS를 우선해, 형태소 규칙·사전 병합이 동일한 데이터를 사용할 수 있게 합니다.

## 사용

```bash
geullint --dictionary-overlay .geullint.overlay docs/
```

여러 팀 사전을 겹칠 수도 있습니다.

```bash
geullint \
  --dictionary-overlay .geullint.overlay \
  --dictionary-overlay terminology.overlay \
  docs/
```

잘못된 헤더나 탭 구분이 없는 항목은 종료 코드 `2`와 함께 거부합니다. 따라서 CI에서도 파일 형식 오류를 조용히 무시하지 않습니다.

## VS Code와 LSP

VS Code 확장도 CLI와 **같은** `geullint-overlay-v1` 파일을 로컬에서 읽을 수 있습니다. 프로젝트의 `.vscode/settings.json`에 파일 경로를 넣으세요.

```json
{
  "geullint.dictionaryOverlayPaths": [".geullint.overlay"]
}
```

상대 경로는 첫 워크스페이스 폴더를 기준으로 해석합니다. 기존 `geullint.dictionaryOverlay` 설정은 표면형을 직접 나열하는 용도로 그대로 유지되며, 두 설정의 표면형은 함께 적용됩니다. 확장과 LSP도 지정한 로컬 overlay 파일만 읽으며 네트워크·API·계정·텔레메트리를 사용하지 않습니다. 파일을 읽거나 형식을 해석할 수 없으면 새 설정을 적용하지 않고 VS Code 출력 채널에 경고를 표시합니다.

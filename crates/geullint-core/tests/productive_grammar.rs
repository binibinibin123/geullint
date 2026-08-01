use geullint_core::{LintConfig, Profile, SourceKind, lint_text};

struct Family {
    rule_id: &'static str,
    safe_fix: bool,
    errors: &'static [(&'static str, &'static str, &'static str)],
    controls: &'static [&'static str],
}

const FAMILIES: &[Family] = &[
    Family {
        rule_id: "grammar.ending.seumnida",
        safe_fix: true,
        errors: &[
            ("보고서를 읽읍니다.", "읽읍니다", "읽습니다"),
            ("안내문을 쓰읍니다.", "쓰읍니다", "씁니다"),
            ("지금 문을 열읍니까?", "열읍니까", "엽니까"),
        ],
        controls: &[
            "보고서를 읽습니다.",
            "안내문을 씁니다.",
            "지금 문을 엽니까?",
            "함께 먹읍시다.",
            "‘읍니다’라는 옛 어미를 설명했다.",
        ],
    },
    Family {
        rule_id: "grammar.ending.sipsio",
        safe_fix: true,
        errors: &[
            ("내용을 확인하십시요.", "확인하십시요", "확인하십시오"),
            ("자리에 앉으십시요.", "앉으십시요", "앉으십시오"),
            ("이쪽으로 오십시요.", "오십시요", "오십시오"),
        ],
        controls: &[
            "내용을 확인하십시오.",
            "자리에 앉으십시오.",
            "이쪽으로 오십시오.",
            "편하게 드십쇼.",
            "십시요원은 작품 속 인물이다.",
        ],
    },
    Family {
        rule_id: "grammar.copula.anieyo",
        safe_fix: true,
        errors: &[
            ("그건 제 것이 아니예요.", "아니예요", "아니에요"),
            ("오늘 마감은 아니예요.", "아니예요", "아니에요"),
            ("제가 고른 답은 아니예요.", "아니예요", "아니에요"),
        ],
        controls: &[
            "그건 제 것이 아니에요.",
            "그 말은 사실이 아니어요.",
            "대답은 아니요입니다.",
            "‘아니에요’라고 답했다.",
            "아니예요법은 가상의 작품명이다.",
        ],
    },
    Family {
        rule_id: "grammar.negation.anh-doe",
        safe_fix: true,
        errors: &[
            ("지금은 접속이 않됩니다.", "않됩니다", "안 됩니다"),
            ("여기서 뛰면 않돼요.", "않돼요", "안 돼요"),
            (
                "비가 오면 출발이 않되니까 기다리자.",
                "않되니까",
                "안 되니까",
            ),
        ],
        controls: &[
            "지금은 접속이 안 됩니다.",
            "여기서 뛰면 안 돼요.",
            "사정이 참 안됐습니다.",
            "그는 대답하지 않았다.",
            "‘않다’의 활용을 배웠다.",
            "무리하지 않되 원칙은 지킨다.",
        ],
    },
    Family {
        rule_id: "spacing.dependent-noun.de",
        safe_fix: false,
        errors: &[
            ("오늘 묵을데가 없다.", "묵을데가", "묵을 데가"),
            ("잠시 쉴데를 찾았다.", "쉴데를", "쉴 데를"),
            ("근처에는 먹을데도 많다.", "먹을데도", "먹을 데도"),
        ],
        controls: &[
            "오늘 묵을 데가 없다.",
            "그 말은 쓸데가 없다.",
            "두 군데를 살폈다.",
            "현대가 세운 건물이다.",
            "김데가는 소설 속 인물이다.",
        ],
    },
    Family {
        rule_id: "spacing.dependent-noun.chae",
        safe_fix: false,
        errors: &[
            ("외투를 입은채로 잠들었다.", "입은채로", "입은 채로"),
            ("문을 닫은채로 떠났다.", "닫은채로", "닫은 채로"),
            ("모두가 선채로 박수쳤다.", "선채로", "선 채로"),
        ],
        controls: &[
            "외투를 입은 채로 잠들었다.",
            "산채로 만든 반찬이다.",
            "집 한 채로 충분하다.",
            "채로 흙을 걸렀다.",
            "김은채가 발표했다.",
        ],
    },
    Family {
        rule_id: "spacing.dependent-noun.deut",
        safe_fix: false,
        errors: &[
            ("그는 모르는듯하다.", "모르는듯하다", "모르는 듯하다"),
            ("회의가 끝난듯싶다.", "끝난듯싶다", "끝난 듯싶다"),
            ("아이가 금방 울듯이 보였다.", "울듯이", "울 듯이"),
        ],
        controls: &[
            "그는 모르는 듯하다.",
            "제법 그럴듯하다.",
            "옷차림이 번듯하다.",
            "줄을 반듯하게 그었다.",
            "김듯이는 가상의 인물이다.",
        ],
    },
    Family {
        rule_id: "spacing.dependent-noun.mankeum",
        safe_fix: false,
        errors: &[
            ("먹을만큼 덜어 가세요.", "먹을만큼", "먹을 만큼"),
            ("먹은만큼 값을 내세요.", "먹은만큼", "먹은 만큼"),
            (
                "노력하는만큼 결과가 따른다.",
                "노력하는만큼",
                "노력하는 만큼",
            ),
        ],
        controls: &[
            "먹을 만큼 덜어 가세요.",
            "마을만큼 조용한 곳이다.",
            "가을만큼 선선한 계절이다.",
            "노을만큼 붉은 빛이다.",
            "그만큼 준비했다.",
        ],
    },
    Family {
        rule_id: "spacing.dependent-noun.daero",
        safe_fix: false,
        errors: &[
            ("설명을 들은대로 적었다.", "들은대로", "들은 대로"),
            ("화면에 보이는대로 누르세요.", "보이는대로", "보이는 대로"),
            ("먹을대로 먹고 출발했다.", "먹을대로", "먹을 대로"),
        ],
        controls: &[
            "설명을 들은 대로 적었다.",
            "마음대로 고르세요.",
            "차례대로 입장했다.",
            "원칙대로 처리했다.",
            "그대로 두세요.",
        ],
    },
    Family {
        rule_id: "spacing.dependent-noun.beop",
        safe_fix: false,
        errors: &[
            ("사람은 실수하면서 사는법이다.", "사는법이다", "사는 법이다"),
            ("이 문제를 푸는법입니다.", "푸는법입니다", "푸는 법입니다"),
            ("도구를 익히는법이 따로 있다.", "익히는법이", "익히는 법이"),
        ],
        controls: &[
            "사람은 실수하면서 사는 법이다.",
            "그 행위는 불법이다.",
            "헌법입니다.",
            "새 방법이 필요하다.",
            "법이 허용하는 범위다.",
        ],
    },
    Family {
        rule_id: "spacing.dependent-noun.ri",
        safe_fix: false,
        errors: &[
            ("그가 약속을 잊을리가 없다.", "잊을리가", "잊을 리가"),
            ("이 문서를 읽을리는 없다.", "읽을리는", "읽을 리는"),
            ("그 사실을 모를리도 없다.", "모를리도", "모를 리도"),
        ],
        controls: &[
            "그가 약속을 잊을 리가 없다.",
            "그에게는 권리가 없다.",
            "설명에 논리가 없다.",
            "그리 멀리도 없다.",
            "물리가 없는 교육 과정이다.",
        ],
    },
];

fn diagnostics_for(text: &str, source_kind: SourceKind) -> Vec<geullint_core::Diagnostic> {
    lint_text(
        text,
        source_kind,
        &LintConfig {
            profile: Profile::Default,
            ..LintConfig::default()
        },
    )
}

#[test]
fn productive_families_cover_distinct_errors_with_utf8_ranges_and_idempotent_suggestions() {
    for family in FAMILIES {
        assert!(family.errors.len() >= 3, "{} error cases", family.rule_id);
        for &(text, original, suggestion) in family.errors {
            let matches: Vec<_> = diagnostics_for(text, SourceKind::PlainText)
                .into_iter()
                .filter(|diagnostic| diagnostic.rule_id == family.rule_id)
                .collect();
            assert_eq!(matches.len(), 1, "{}: {text}", family.rule_id);
            let diagnostic = &matches[0];
            assert_eq!(diagnostic.original, original, "{}: {text}", family.rule_id);
            assert_eq!(
                diagnostic.suggestions,
                [suggestion],
                "{}: {text}",
                family.rule_id
            );
            assert_eq!(
                diagnostic.safe_fix, family.safe_fix,
                "{}: {text}",
                family.rule_id
            );
            assert_eq!(
                &text[diagnostic.range.start..diagnostic.range.end],
                original
            );

            let mut corrected = text.to_owned();
            corrected.replace_range(
                diagnostic.range.start..diagnostic.range.end,
                &diagnostic.suggestions[0],
            );
            assert!(
                diagnostics_for(&corrected, SourceKind::PlainText)
                    .iter()
                    .all(|item| item.rule_id != family.rule_id),
                "{} recurs after {corrected}",
                family.rule_id
            );
        }
    }
}

#[test]
fn productive_families_preserve_normal_lexical_and_named_forms() {
    for family in FAMILIES {
        assert!(family.controls.len() >= 5, "{} controls", family.rule_id);
        for &text in family.controls {
            assert!(
                diagnostics_for(text, SourceKind::PlainText)
                    .iter()
                    .all(|diagnostic| diagnostic.rule_id != family.rule_id),
                "{} false positive: {text}",
                family.rule_id
            );
        }
    }
}

#[test]
fn productive_families_skip_code_strings_and_check_comments() {
    for family in FAMILIES {
        let (text, _, _) = family.errors[0];
        let string_source = format!(r#"const sample = "{text}";"#);
        assert!(
            diagnostics_for(&string_source, SourceKind::TypeScript)
                .iter()
                .all(|diagnostic| diagnostic.rule_id != family.rule_id),
            "{} inspected a string literal",
            family.rule_id
        );

        let comment_source = format!("// {text}");
        assert!(
            diagnostics_for(&comment_source, SourceKind::TypeScript)
                .iter()
                .any(|diagnostic| diagnostic.rule_id == family.rule_id),
            "{} skipped a source comment",
            family.rule_id
        );
    }
}

#[test]
fn contextual_spacing_does_not_read_past_a_source_comment_boundary() {
    let source = "// 그가 잊을리가\n없다();";

    assert!(
        diagnostics_for(source, SourceKind::TypeScript)
            .iter()
            .all(|diagnostic| diagnostic.rule_id != "spacing.dependent-noun.ri")
    );
}

use geullint_core::{
    Confidence, DictionaryOverlay, DocumentSession, Engine, FixSafety, LintConfig, Profile,
    RulePack, Severity, SourceKind, TextEdit, apply_safe_fixes, available_rule_ids, lint_text,
    rule_metadata,
};

#[cfg(feature = "morphology")]
use geullint_core::MorphAnalyzer;

#[test]
fn reports_a_safe_spelling_fix_with_the_original_utf8_range() {
    let diagnostics = lint_text(
        "오늘이 몇일이지?",
        SourceKind::PlainText,
        &LintConfig::default(),
    );

    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.rule_id, "spelling.lexical.myeochil");
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(diagnostic.original, "몇일");
    assert_eq!(diagnostic.suggestions, ["며칠"]);
    assert!(diagnostic.safe_fix);
    assert_eq!(
        &"오늘이 몇일이지?"[diagnostic.range.start..diagnostic.range.end],
        "몇일"
    );
}

#[test]
fn honours_disabled_rule_ids() {
    let config = LintConfig {
        disabled_rules: vec!["spelling.lexical.myeochil".into()],
        ..LintConfig::default()
    };

    let diagnostics = lint_text("몇일 뒤에 만나요.", SourceKind::PlainText, &config);

    assert!(diagnostics.is_empty());
}

#[test]
fn scans_markdown_prose_but_not_fenced_or_inline_code() {
    let diagnostics = lint_text(
        "몇일 뒤에 만나요.\n\n```js\nconst label = '몇일';\n```\n\n`몇일`은 코드입니다.",
        SourceKind::Markdown,
        &LintConfig::default(),
    );

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].original, "몇일");
}

#[test]
fn scans_code_comments_but_not_string_literals() {
    let diagnostics = lint_text(
        "const label = '몇일'; // 몇일\n/* 몇일 */",
        SourceKind::JavaScript,
        &LintConfig::default(),
    );

    assert_eq!(diagnostics.len(), 2);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.original == "몇일")
    );
}

#[test]
fn applies_the_curated_yaml_lexical_rules() {
    let diagnostics = lint_text(
        "금새 어의없는 역활을 맡았다.",
        SourceKind::PlainText,
        &LintConfig::default(),
    );
    let rule_ids: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_id.as_str())
        .collect();

    assert_eq!(
        rule_ids,
        [
            "spelling.lexical.geumse",
            "spelling.lexical.eoieopda",
            "spelling.lexical.yeokhal",
        ]
    );
    assert_eq!(diagnostics[0].suggestions, ["금세"]);
}

#[test]
fn applies_context_sensitive_grammar_spacing_and_punctuation_rules() {
    let diagnostics = lint_text(
        "일이 되서 할려고 책를 봤다. 검토하는중이다. 끝났다.다음",
        SourceKind::PlainText,
        &LintConfig::default(),
    );
    let rule_ids: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_id.as_str())
        .collect();

    assert_eq!(
        rule_ids,
        [
            "grammar.conjugation.doe-to-dwae",
            "grammar.ending.euryeo",
            "spacing.dependent-noun.jung",
            "punctuation.space-after-sentence-mark",
        ]
    );
    assert_eq!(diagnostics[0].suggestions, ["돼서"]);
}

#[test]
fn does_not_flag_correct_particle_forms() {
    let diagnostics = lint_text(
        "책은 사과가 필요하고, 친구와 집으로 간다.",
        SourceKind::PlainText,
        &LintConfig::default(),
    );

    assert!(diagnostics.is_empty());
}

#[test]
fn does_not_mistake_the_lexical_form_eoieopda_for_a_subject_particle() {
    let diagnostics = lint_text(
        "그는 어이없는 표정을 지었다.",
        SourceKind::PlainText,
        &LintConfig::default(),
    );

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_id != "grammar.particle.subject-allomorph")
    );
}

#[test]
fn finds_parallel_choice_particles_duplicate_particles_and_repeated_words() {
    let diagnostics = lint_text(
        "커피던지 차던지 고르세요. 문서를를 문서를 문서를 저장합니다.",
        SourceKind::PlainText,
        &LintConfig::default(),
    );
    let rule_ids: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_id.as_str())
        .collect();

    assert_eq!(
        rule_ids,
        [
            "grammar.ending.deun-choice",
            "grammar.ending.deun-choice",
            "grammar.particle.duplicate",
            "repetition.adjacent-word",
        ]
    );
    assert_eq!(diagnostics[0].suggestions, ["커피든지"]);
    assert_eq!(diagnostics[2].suggestions, ["를"]);
    assert!(!diagnostics[3].safe_fix);
}

#[test]
fn engine_rechecks_a_document_after_a_utf8_safe_edit() {
    let engine = Engine::new(LintConfig::default());
    let mut document = DocumentSession::new("몇일 뒤에 만나요.", SourceKind::PlainText);

    assert_eq!(engine.check_document(&document).len(), 1);
    document
        .apply_edit(&TextEdit {
            range: geullint_core::TextRange { start: 0, end: 6 },
            replacement: "며칠".into(),
        })
        .expect("valid UTF-8 edit");

    assert_eq!(document.text(), "며칠 뒤에 만나요.");
    assert!(engine.check_document(&document).is_empty());
}

#[test]
fn ships_the_curated_rule_ids() {
    let rule_ids = available_rule_ids();

    assert!(rule_ids.contains("spelling.lexical.myeochil"));
    assert!(rule_ids.contains("grammar.particle.object-allomorph"));
    assert!(rule_ids.contains("repetition.adjacent-word"));
}

#[test]
fn exposes_stable_metadata_for_each_rule() {
    let metadata = rule_metadata("spelling.lexical.myeochil").expect("bundled rule metadata");

    assert_eq!(metadata.category, "spelling");
    assert_eq!(metadata.confidence, Confidence::High);
    assert_eq!(metadata.fix_safety, FixSafety::Safe);
    assert!(metadata.profiles.contains(&Profile::Default));
    assert!(metadata.documentation_url.starts_with("https://"));
}

#[test]
fn user_dictionary_only_suppresses_dictionary_aware_lexical_rules() {
    let config = LintConfig {
        user_dictionary: vec!["몇일".into()],
        ..LintConfig::default()
    };

    let diagnostics = lint_text("몇일 뒤에 되서 만나요.", SourceKind::PlainText, &config);
    let rule_ids: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_id.as_str())
        .collect();

    assert_eq!(rule_ids, ["grammar.conjugation.doe-to-dwae"]);
}

#[test]
fn applies_only_safe_non_overlapping_fixes() {
    let source = "몇일 문서를 문서를 저장합니다.";
    let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());

    assert_eq!(
        apply_safe_fixes(source, &diagnostics),
        "며칠 문서를 문서를 저장합니다."
    );
}

#[test]
fn loads_a_versioned_dictionary_overlay_without_network_access() {
    let overlay = DictionaryOverlay::parse("geullint-overlay-v1\n며칠\tNNG\n봬요\tVV\n")
        .expect("valid overlay");

    assert_eq!(overlay.part_of_speech("며칠"), Some("NNG"));
    assert_eq!(overlay.part_of_speech("봬요"), Some("VV"));
    assert_eq!(overlay.part_of_speech("없는말"), None);
    assert_eq!(overlay.entry_count(), 2);
    assert!(DictionaryOverlay::parse("며칠\tNNG\n").is_err());
}

#[test]
fn applies_overlay_terms_to_dictionary_aware_lexical_diagnostics() {
    let overlay = DictionaryOverlay::parse("geullint-overlay-v1\n몇일\tNNP\n")
        .expect("valid project overlay");
    let config = LintConfig {
        dictionary_overlay: overlay.surfaces().map(str::to_owned).collect(),
        ..LintConfig::default()
    };

    let diagnostics = lint_text("몇일 뒤에 만나요.", SourceKind::PlainText, &config);

    assert!(diagnostics.is_empty());
}

#[test]
#[cfg(feature = "morphology")]
fn analyzes_korean_text_with_the_bundled_morphology_dictionary() {
    let analyzer = MorphAnalyzer::bundled().expect("bundled Korean morphology dictionary");
    let tokens = analyzer.analyze("책을 읽는다.").expect("Korean analysis");

    assert!(tokens.iter().any(|token| token.surface == "책"));
    assert!(
        tokens
            .iter()
            .any(|token| token.part_of_speech.starts_with("JKO"))
    );
}

#[test]
#[cfg(feature = "morphology")]
fn analyzed_document_preserves_only_source_safe_morphology_tokens() {
    let source = "책을 읽는다. `책을 숨긴다`";
    let document = geullint_core::AnalyzedDocument::new(source, SourceKind::Markdown);
    let tokens = document.morphology_tokens();

    assert!(tokens.len() > document.words().len());
    assert!(tokens.iter().any(|token| {
        token.surface == "책" && token.range == geullint_core::TextRange { start: 0, end: 3 }
    }));
    assert!(tokens.iter().any(|token| {
        token.surface == "을" && token.range == geullint_core::TextRange { start: 3, end: 6 }
    }));
    assert_eq!(
        tokens.iter().filter(|token| token.surface == "책").count(),
        1,
        "the token inside Markdown code must be excluded"
    );
    for token in tokens {
        assert_eq!(
            &source[token.range.start..token.range.end],
            token.surface,
            "every morphology range must point into the original UTF-8 source"
        );
    }
}

#[test]
#[cfg(feature = "morphology")]
fn applies_overlay_pos_tags_to_bundled_morphology_results() {
    let overlay =
        DictionaryOverlay::parse("geullint-overlay-v1\n며칠\tNNP\n").expect("valid overlay");
    let analyzer = MorphAnalyzer::with_overlay(overlay).expect("bundled Korean morphology");
    let tokens = analyzer
        .analyze("며칠 뒤에 만나요.")
        .expect("Korean analysis");

    assert!(
        tokens
            .iter()
            .any(|token| token.surface == "며칠" && token.part_of_speech == "NNP")
    );
}

#[test]
fn only_enables_editorial_rules_for_the_editorial_profile() {
    let source = "그것은 가장 최고다.";
    let default_diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());
    let strict_diagnostics = lint_text(
        source,
        SourceKind::PlainText,
        &LintConfig {
            profile: Profile::Strict,
            ..LintConfig::default()
        },
    );
    let editorial_diagnostics = lint_text(
        source,
        SourceKind::PlainText,
        &LintConfig {
            profile: Profile::Editorial,
            ..LintConfig::default()
        },
    );

    assert!(default_diagnostics.is_empty());
    assert!(strict_diagnostics.is_empty());
    assert_eq!(
        editorial_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.rule_id.as_str())
            .collect::<Vec<_>>(),
        ["style.redundancy.gajang-choego"]
    );
}

#[test]
fn publishes_the_minimum_profile_in_rule_metadata() {
    let metadata =
        rule_metadata("style.redundancy.gajang-choego").expect("editorial rule metadata");
    let repetition_metadata =
        rule_metadata("repetition.adjacent-word").expect("repetition rule metadata");

    assert_eq!(metadata.profiles, [Profile::Editorial]);
    assert_eq!(metadata.fix_safety, FixSafety::Review);
    assert_eq!(repetition_metadata.fix_safety, FixSafety::Review);
}

#[test]
fn applies_a_versioned_local_rule_pack_without_network_access() {
    let pack = RulePack::parse(
        r#"
version: 1
language: ko
rules:
  - id: spelling.project.typo
    severity: warning
    message: "프로젝트 표기를 확인하세요."
    safeFix: true
    replacements:
      - from: 글린트
        to: GeulLint
"#,
    )
    .expect("valid versioned local rule pack");
    let engine = Engine::with_rule_packs(LintConfig::default(), vec![pack])
        .expect("pack IDs do not collide with bundled rules");

    let diagnostics = engine.check("글린트를 실행합니다.", SourceKind::PlainText);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].rule_id, "spelling.project.typo");
    assert_eq!(diagnostics[0].suggestions, ["GeulLint"]);
}

#[test]
fn bundled_lexical_rules_match_at_eojeol_starts_not_inside_other_words() {
    let source = "몇일이나 기다렸지만 황금새우만 남았다.";
    let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].rule_id, "spelling.lexical.myeochil");
    assert_eq!(diagnostics[0].original, "몇일");
    assert_eq!(
        &source[diagnostics[0].range.start..diagnostics[0].range.end],
        "몇일"
    );
}

#[test]
fn local_rule_pack_boundaries_are_explicit_and_utf8_safe() {
    let pack = RulePack::parse(
        r#"
version: 1
language: ko
rules:
  - id: project.prefix
    severity: warning
    message: "어절 시작에서만 검사합니다."
    safeFix: true
    replacements:
      - from: 글린트
        to: GeulLint
        boundary: prefix
  - id: project.word
    severity: warning
    message: "독립된 어절만 검사합니다."
    safeFix: true
    replacements:
      - from: 맞춤
        to: 바꿈
        boundary: word
"#,
    )
    .expect("valid boundary-aware pack");
    let engine = Engine::with_rule_packs(LintConfig::default(), vec![pack])
        .expect("pack IDs do not collide with bundled rules");
    let source = "한글린트 글린트로 맞춤 맞춤법";

    let diagnostics = engine.check(source, SourceKind::PlainText);

    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.rule_id.as_str(), diagnostic.original.as_str()))
            .collect::<Vec<_>>(),
        [("project.prefix", "글린트"), ("project.word", "맞춤")]
    );
    assert!(diagnostics.iter().all(|diagnostic| {
        source[diagnostic.range.start..diagnostic.range.end] == diagnostic.original
    }));
}

#[test]
fn legacy_v1_rule_pack_without_boundary_keeps_substring_matching() {
    let pack = RulePack::parse(
        r#"
version: 1
language: ko
rules:
  - id: project.legacy-substring
    severity: warning
    message: "기존 v1 규칙의 부분 문자열 동작"
    safeFix: true
    replacements:
      - from: 린트
        to: Lint
"#,
    )
    .expect("valid legacy v1 pack");
    let engine = Engine::with_rule_packs(LintConfig::default(), vec![pack])
        .expect("pack IDs do not collide with bundled rules");

    let diagnostics = engine.check("글린트를 실행합니다.", SourceKind::PlainText);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].rule_id, "project.legacy-substring");
    assert_eq!(diagnostics[0].original, "린트");
}

#[test]
fn bundled_boundaries_keep_suffix_errors_without_lexical_internal_matches() {
    let source = "내가 말할께. 아직 안됬다. 황금새우를 먹었다.";

    let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());

    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.rule_id.as_str(), diagnostic.original.as_str()))
            .collect::<Vec<_>>(),
        [
            ("grammar.ending.hal-ge", "할께"),
            ("spelling.conjugation.dwaet", "됬"),
        ]
    );
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_id != "spelling.lexical.geumse")
    );
}

#[test]
fn bundled_suffix_fragments_remain_matchable_before_following_endings() {
    let source = "내가 말할께요. 아직 안됬다고 말했다.";

    let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());

    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.rule_id.as_str(), diagnostic.original.as_str()))
            .collect::<Vec<_>>(),
        [
            ("grammar.ending.hal-ge", "할께"),
            ("spelling.conjugation.dwaet", "됬"),
        ]
    );
}

#[test]
fn bundled_spacing_fragments_match_inside_eojeols_without_flagging_correct_spacing() {
    let incorrect = lint_text(
        "그것뿐만아니라 다른 방법도 있다.",
        SourceKind::PlainText,
        &LintConfig::default(),
    );
    let correct = lint_text(
        "그것뿐만 아니라 다른 방법도 있다.",
        SourceKind::PlainText,
        &LintConfig::default(),
    );

    assert_eq!(
        incorrect
            .iter()
            .map(|diagnostic| (diagnostic.rule_id.as_str(), diagnostic.original.as_str()))
            .collect::<Vec<_>>(),
        [("spacing.fixed.ppunman-anira", "뿐만아니라")]
    );
    assert!(
        correct
            .iter()
            .all(|diagnostic| diagnostic.rule_id != "spacing.fixed.ppunman-anira")
    );
}

#[test]
fn overlapping_local_patterns_have_stable_source_order() {
    let pack = RulePack::parse(
        r#"
version: 1
language: ko
rules:
  - id: project.long
    severity: warning
    message: "긴 패턴"
    safeFix: true
    replacements:
      - from: 가나다
        to: 긴교정
        boundary: substring
  - id: project.short
    severity: warning
    message: "겹친 패턴"
    safeFix: true
    replacements:
      - from: 나다
        to: 짧은교정
        boundary: substring
"#,
    )
    .expect("valid overlapping pack");
    let engine = Engine::with_rule_packs(LintConfig::default(), vec![pack])
        .expect("pack IDs do not collide with bundled rules");
    let source = "앞 가나다 뒤";

    let diagnostics = engine.check(source, SourceKind::PlainText);

    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.rule_id.as_str(), diagnostic.range))
            .collect::<Vec<_>>(),
        [
            (
                "project.long",
                geullint_core::TextRange { start: 4, end: 13 }
            ),
            (
                "project.short",
                geullint_core::TextRange { start: 7, end: 13 }
            ),
        ]
    );
}

#[test]
fn bundled_pattern_does_not_report_self_overlapping_occurrences() {
    let diagnostics = lint_text(",,,", SourceKind::PlainText, &LintConfig::default());

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].rule_id, "punctuation.duplicate.comma");
    assert_eq!(
        diagnostics[0].range,
        geullint_core::TextRange { start: 0, end: 3 }
    );
    assert_eq!(diagnostics[0].original, ",,,");
}

#[test]
fn punctuation_runs_produce_one_composed_safe_fix_each() {
    let source = "사과,,,,배를 샀다. 설명을 마쳤다  .다음 문장이다.";
    let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());
    let punctuation = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_id.starts_with("punctuation."))
        .map(|diagnostic| {
            (
                diagnostic.rule_id.as_str(),
                diagnostic.original.as_str(),
                diagnostic.suggestions[0].as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        punctuation,
        [
            ("punctuation.duplicate.comma", ",,,,", ", "),
            ("punctuation.no-space-before-mark", "  ", ""),
            ("punctuation.space-after-sentence-mark", ".", ". "),
        ]
    );
    assert_eq!(
        apply_safe_fixes(source, &diagnostics),
        "사과, 배를 샀다. 설명을 마쳤다. 다음 문장이다."
    );
}

#[test]
fn rejects_invalid_or_colliding_local_rule_packs() {
    let unsupported_version = RulePack::parse(
        r"
version: 2
language: ko
rules: []
",
    );
    assert!(unsupported_version.is_err());

    let colliding_pack = RulePack::parse(
        r#"
version: 1
language: ko
rules:
  - id: spelling.lexical.myeochil
    severity: error
    message: "충돌"
    safeFix: true
    replacements:
      - from: 오표기
        to: 바른표기
"#,
    )
    .expect("syntactically valid pack");
    assert!(Engine::with_rule_packs(LintConfig::default(), vec![colliding_pack]).is_err());
}

#[test]
fn analyzes_only_lintable_korean_words_and_preserves_utf8_ranges() {
    let source = "본문 돼게 `코드 돼게`";
    let document = geullint_core::AnalyzedDocument::new(source, SourceKind::Markdown);

    assert_eq!(document.words().len(), 2);
    assert_eq!(document.words()[0].surface, "본문");
    assert_eq!(document.words()[1].surface, "돼게");
    assert_eq!(
        &source[document.words()[1].range.start..document.words()[1].range.end],
        "돼게"
    );
}

#[test]
fn corrects_contextual_korean_endings_without_touching_valid_words() {
    let source = "안녕하세요 감사해용 웬만하면 돼게 할려고 하였다";
    let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());

    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.rule_id.as_str())
            .collect::<Vec<_>>(),
        ["grammar.conjugation.dwae-to-doe", "grammar.ending.euryeo",]
    );
    assert_eq!(
        apply_safe_fixes(source, &diagnostics),
        "안녕하세요 감사해용 웬만하면 되게 하려고 하였다"
    );
}

#[test]
fn corrects_each_supported_dwae_suffix() {
    for (source, suggestion) in [
        ("돼게", "되게"),
        ("안돼게요", "안되게요"),
        ("돼면", "되면"),
        ("돼고", "되고"),
        ("돼는", "되는"),
        ("돼겠", "되겠"),
        ("진행돼면서", "진행되면서"),
        ("적용돼도록", "적용되도록"),
    ] {
        let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());

        assert_eq!(diagnostics.len(), 1, "{source}");
        assert_eq!(diagnostics[0].rule_id, "grammar.conjugation.dwae-to-doe");
        assert_eq!(diagnostics[0].original, source);
        assert_eq!(diagnostics[0].suggestions, [suggestion]);
        assert!(diagnostics[0].safe_fix);
    }
}

#[test]
fn preserves_audited_dwae_controls() {
    for source in [
        "돼서",
        "돼도",
        "돼요",
        "돼야",
        "돼지",
        "돼지만",
        "돼지도",
        "돼고기",
    ] {
        let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());
        assert!(
            diagnostics
                .iter()
                .all(|item| item.rule_id != "grammar.conjugation.dwae-to-doe"),
            "{source}: {diagnostics:?}"
        );
    }
}

#[test]
fn safely_corrects_malformed_contractions_with_utf8_ranges() {
    let source = "ASCII😀 됀다면 됄까요 됌을 확인한다";
    let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());
    let contractions: Vec<_> = diagnostics
        .iter()
        .filter(|item| item.rule_id == "grammar.conjugation.doe-to-dwae")
        .collect();

    assert_eq!(contractions.len(), 3);
    assert_eq!(
        contractions
            .iter()
            .map(|item| item.original.as_str())
            .collect::<Vec<_>>(),
        ["됀", "됄", "됌"]
    );
    assert_eq!(
        contractions
            .iter()
            .map(|item| item.suggestions[0].as_str())
            .collect::<Vec<_>>(),
        ["된", "될", "됨"]
    );
    assert!(contractions.iter().all(|item| item.safe_fix));
    for diagnostic in contractions {
        assert_eq!(
            &source[diagnostic.range.start..diagnostic.range.end],
            diagnostic.original
        );
    }
    assert_eq!(
        apply_safe_fixes(source, &diagnostics),
        "ASCII😀 된다면 될까요 됨을 확인한다"
    );
}

#[test]
fn preserves_and_safely_corrects_each_doe_to_dwae_form() {
    for (source, expected) in [
        ("안되서요", "안돼서요"),
        ("해도 되도", "해도 돼도"),
        ("잘되요", "잘돼요"),
        ("준비되야", "준비돼야"),
        ("됀", "된"),
        ("됄까요", "될까요"),
        ("됌을", "됨을"),
    ] {
        let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());
        let matching: Vec<_> = diagnostics
            .iter()
            .filter(|item| item.rule_id == "grammar.conjugation.doe-to-dwae")
            .collect();
        assert_eq!(matching.len(), 1, "{source}: {diagnostics:?}");
        assert!(matching[0].safe_fix, "{source}");
        assert_eq!(apply_safe_fixes(source, &diagnostics), expected, "{source}");
    }
}

#[test]
fn doe_to_dwae_preserves_do_stems_and_is_idempotent() {
    let controls = "되도록 최선을 다했다. 되돌리다. 되돌아가다.";
    let control_diagnostics = lint_text(controls, SourceKind::PlainText, &LintConfig::default());
    assert!(
        control_diagnostics
            .iter()
            .all(|item| item.rule_id != "grammar.conjugation.doe-to-dwae"),
        "{control_diagnostics:?}"
    );
    assert_eq!(apply_safe_fixes(controls, &control_diagnostics), controls);

    let standalone = "해도 되도";
    let standalone_diagnostics =
        lint_text(standalone, SourceKind::PlainText, &LintConfig::default());
    assert_eq!(
        apply_safe_fixes(standalone, &standalone_diagnostics),
        "해도 돼도"
    );

    let mixed = "되도 괜찮지만 되도록 되돌리지는 마라.";
    let first_diagnostics = lint_text(mixed, SourceKind::PlainText, &LintConfig::default());
    let fixed_once = apply_safe_fixes(mixed, &first_diagnostics);
    let second_diagnostics = lint_text(&fixed_once, SourceKind::PlainText, &LintConfig::default());
    let fixed_twice = apply_safe_fixes(&fixed_once, &second_diagnostics);

    assert_eq!(fixed_once, "돼도 괜찮지만 되도록 되돌리지는 마라.");
    assert_eq!(fixed_twice, fixed_once);
}

#[test]
fn safe_fixes_compose_around_the_same_sentence_mark_in_one_pass() {
    let source = "어쨋든 자료를를 확인하십시요 .다음 회의는 금새 시작됬다.";
    let expected = "어쨌든 자료를 확인하십시오. 다음 회의는 금세 시작됐다.";
    let config = LintConfig {
        profile: Profile::Editorial,
        ..LintConfig::default()
    };

    let diagnostics = lint_text(source, SourceKind::PlainText, &config);
    let fixed_once = apply_safe_fixes(source, &diagnostics);
    let fixed_twice = apply_safe_fixes(
        &fixed_once,
        &lint_text(&fixed_once, SourceKind::PlainText, &config),
    );

    assert_eq!(fixed_once, expected);
    assert_eq!(fixed_twice, expected);
}

#[test]
fn engine_safe_fix_reaches_a_stable_result_for_chained_rules() {
    let engine = Engine::new(LintConfig::default());
    let source = "사과,,배를 사고 문서,,,,초안을 검토했다. 설명을 마쳤다  .다음 문장을 썼다. 아이들이 들어오면 않돼게 문을 잠갔다. 문서가 않됬다고 보고했다. 처리가 잘 됬읍니다. 확인이 됬읍니까?";
    let expected = "사과, 배를 사고 문서, 초안을 검토했다. 설명을 마쳤다. 다음 문장을 썼다. 아이들이 들어오면 안 되게 문을 잠갔다. 문서가 안 됐다고 보고했다. 처리가 잘 됐습니다. 확인이 됐습니까?";

    let fixed = engine.fix(source, SourceKind::PlainText);

    assert_eq!(fixed, expected);
    assert_eq!(engine.fix(&fixed, SourceKind::PlainText), fixed);
}

#[test]
fn engine_safe_fix_abandons_a_cyclic_local_rule_pack() {
    let pack = RulePack::parse(
        r#"
version: 1
language: ko
rules:
  - id: project.a-to-b
    severity: warning
    message: "A를 B로"
    safeFix: true
    replacements:
      - from: 가나다
        to: 라마바
  - id: project.b-to-a
    severity: warning
    message: "B를 A로"
    safeFix: true
    replacements:
      - from: 라마바
        to: 가나다
"#,
    )
    .expect("valid cyclic local pack");
    let engine = Engine::with_rule_packs(LintConfig::default(), vec![pack])
        .expect("pack IDs do not collide");

    assert_eq!(
        engine.fix("앞 가나다 뒤", SourceKind::PlainText),
        "앞 가나다 뒤"
    );
    assert_eq!(
        engine.fix("앞 라마바 뒤", SourceKind::PlainText),
        "앞 라마바 뒤"
    );
}

#[test]
fn safely_corrects_only_known_euryeo_forms_and_preserves_the_eojeol() {
    for (source, expected) in [
        ("할려고", "하려고"),
        ("할려면", "하려면"),
        ("먹을려고", "먹으려고"),
        ("먹을려면", "먹으려면"),
        ("읽을려고", "읽으려고"),
        ("읽을려면", "읽으려면"),
        ("잡을려고", "잡으려고"),
        ("잡을려면", "잡으려면"),
    ] {
        let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());
        assert_eq!(diagnostics.len(), 1, "{source}: {diagnostics:?}");
        assert_eq!(diagnostics[0].rule_id, "grammar.ending.euryeo");
        assert!(diagnostics[0].safe_fix);
        assert_eq!(apply_safe_fixes(source, &diagnostics), expected, "{source}");
    }

    let source = "재확인할려고도 먹을려면 읽을려고만 잡을려면 좋을려고";
    let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());
    let euryeo: Vec<_> = diagnostics
        .iter()
        .filter(|item| item.rule_id == "grammar.ending.euryeo")
        .collect();

    assert_eq!(euryeo.len(), 4, "{diagnostics:?}");
    assert!(euryeo.iter().all(|item| item.safe_fix));
    assert_eq!(
        apply_safe_fixes(source, &diagnostics),
        "재확인하려고도 먹으려면 읽으려고만 잡으려면 좋을려고"
    );
}

#[test]
fn treats_gal_euryeo_as_review_only_outside_default() {
    let source = "갈려고 갈려면 알려고 밀려고 들려고 그을려고";

    let default_diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());
    assert!(default_diagnostics.iter().all(|item| {
        item.rule_id != "grammar.ending.euryeo-context" && item.rule_id != "grammar.ending.euryeo"
    }));

    for profile in [Profile::Strict, Profile::Editorial] {
        let diagnostics = lint_text(
            source,
            SourceKind::PlainText,
            &LintConfig {
                profile,
                ..LintConfig::default()
            },
        );
        let review: Vec<_> = diagnostics
            .iter()
            .filter(|item| item.rule_id == "grammar.ending.euryeo-context")
            .collect();
        assert_eq!(review.len(), 2, "{profile:?}: {diagnostics:?}");
        assert!(review.iter().all(|item| !item.safe_fix));
        assert_eq!(review[0].suggestions, ["가려고"]);
        assert_eq!(review[1].suggestions, ["가려면"]);
        assert_eq!(apply_safe_fixes(source, &diagnostics), source);
    }
}

#[test]
fn colloquial_yong_is_editorial_review_only_and_conservative() {
    let source = "해용 감사해용 오세용 사용 내용 허용 군용 어용 지용";

    for profile in [Profile::Default, Profile::Strict] {
        let diagnostics = lint_text(
            source,
            SourceKind::PlainText,
            &LintConfig {
                profile,
                ..LintConfig::default()
            },
        );
        assert!(
            diagnostics
                .iter()
                .all(|item| { item.rule_id != "grammar.ending.colloquial-yong" })
        );
    }

    let diagnostics = lint_text(
        source,
        SourceKind::PlainText,
        &LintConfig {
            profile: Profile::Editorial,
            ..LintConfig::default()
        },
    );
    let review: Vec<_> = diagnostics
        .iter()
        .filter(|item| item.rule_id == "grammar.ending.colloquial-yong")
        .collect();
    assert_eq!(review.len(), 3, "{diagnostics:?}");
    assert_eq!(review[0].suggestions, ["해요"]);
    assert_eq!(review[1].suggestions, ["감사해요"]);
    assert_eq!(review[2].suggestions, ["오세요"]);
    assert!(review.iter().all(|item| !item.safe_fix));
    assert_eq!(apply_safe_fixes(source, &diagnostics), source);
}

#[test]
fn duplicate_particle_rule_requires_a_word_final_particle_pair() {
    let normal = "꽃향기가 은은하다. 그는 느긋하게 걷는는커녕 잠시 멈춰 섰다.";
    let normal_diagnostics = lint_text(
        normal,
        SourceKind::PlainText,
        &LintConfig {
            profile: Profile::Editorial,
            ..LintConfig::default()
        },
    );
    assert!(
        normal_diagnostics
            .iter()
            .all(|item| item.rule_id != "grammar.particle.duplicate"),
        "{normal_diagnostics:?}"
    );
    assert_eq!(apply_safe_fixes(normal, &normal_diagnostics), normal);

    let error = "빛은은 부드럽고 자료를를 다시 확인했다.";
    let diagnostics = lint_text(error, SourceKind::PlainText, &LintConfig::default());
    let duplicates = diagnostics
        .iter()
        .filter(|item| item.rule_id == "grammar.particle.duplicate")
        .collect::<Vec<_>>();
    assert_eq!(
        duplicates
            .iter()
            .map(|item| item.original.as_str())
            .collect::<Vec<_>>(),
        ["은은", "를를"]
    );
    assert_eq!(
        apply_safe_fixes(error, &diagnostics),
        "빛은 부드럽고 자료를 다시 확인했다."
    );
}

#[test]
fn ending_rules_skip_markdown_code_and_keep_original_ranges() {
    let source = "😀 안돼게요 `진행돼면서 먹을려고 됀다면` 재확인할려고도";
    let diagnostics = lint_text(source, SourceKind::Markdown, &LintConfig::default());

    assert_eq!(diagnostics.len(), 2, "{diagnostics:?}");
    assert_eq!(diagnostics[0].original, "안돼게요");
    assert_eq!(diagnostics[1].original, "재확인할려고도");
    for diagnostic in &diagnostics {
        assert_eq!(
            &source[diagnostic.range.start..diagnostic.range.end],
            diagnostic.original
        );
    }
    assert_eq!(
        apply_safe_fixes(source, &diagnostics),
        "😀 안되게요 `진행돼면서 먹을려고 됀다면` 재확인하려고도"
    );
}

#[test]
fn does_not_treat_common_yong_words_as_colloquial_endings() {
    let diagnostics = lint_text(
        "사용 용량 내용 허용",
        SourceKind::PlainText,
        &LintConfig::default(),
    );

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_id != "grammar.ending.colloquial-yong")
    );
}

#[test]
fn default_profile_preserves_plausible_ro_words() {
    let source = "근로 환경 진로를 선택했다 난로를 켰다 원로 배우 선로를 점검했다 폭로 기사";
    let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
    assert_eq!(apply_safe_fixes(source, &diagnostics), source);
}

#[test]
fn default_profile_does_not_safely_rewrite_ambiguous_jjige() {
    let source = "감자를 푹 찌게 물을 올렸다.";
    let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());

    assert!(
        diagnostics.iter().all(|diagnostic| !diagnostic.safe_fix),
        "unexpected safe diagnostics: {diagnostics:?}"
    );
    assert_eq!(apply_safe_fixes(source, &diagnostics), source);
}

#[test]
fn strict_profile_keeps_particle_suggestions_review_only() {
    let config = LintConfig {
        profile: Profile::Strict,
        ..LintConfig::default()
    };

    for (source, rule_id, suggestion) in [
        ("책는 유용하다.", "grammar.particle.topic-allomorph", "책은"),
        (
            "나무이 필요하다.",
            "grammar.particle.subject-allomorph",
            "나무가",
        ),
        ("책를 읽는다.", "grammar.particle.object-allomorph", "책을"),
        (
            "책와 연필을 챙긴다.",
            "grammar.particle.comitative-allomorph",
            "책과",
        ),
        (
            "책로 공부한다.",
            "grammar.particle.instrumental-allomorph",
            "책으로",
        ),
    ] {
        let diagnostics = lint_text(source, SourceKind::PlainText, &config);
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.rule_id == rule_id)
            .unwrap_or_else(|| panic!("{rule_id} must review {source:?}"));

        assert_eq!(diagnostic.suggestions, [suggestion]);
        assert!(!diagnostic.safe_fix, "{rule_id} must require review");
        assert_eq!(apply_safe_fixes(source, &diagnostics), source);
    }
}

#[test]
fn strict_profile_keeps_ambiguous_jjige_suggestion_review_only() {
    let source = "감자를 푹 찌게 물을 올렸다.";
    let diagnostics = lint_text(
        source,
        SourceKind::PlainText,
        &LintConfig {
            profile: Profile::Strict,
            ..LintConfig::default()
        },
    );
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.rule_id == "spelling.lexical.jjigae")
        .expect("strict profile keeps the ambiguous spelling review");

    assert_eq!(diagnostic.suggestions, ["찌개"]);
    assert!(!diagnostic.safe_fix);
    assert_eq!(apply_safe_fixes(source, &diagnostics), source);
}

#[test]
fn catches_common_spelling_errors_in_unrelated_sentences() {
    let cases = [
        (
            "회의 자료 데이타를 다시 만들었다.",
            "회의 자료 데이터를 다시 만들었다.",
        ),
        ("그 배우의 설레임이 전해졌다.", "그 배우의 설렘이 전해졌다."),
        ("내노라하는 전문가가 모였다.", "내로라하는 전문가가 모였다."),
        ("왠만하면 오늘 끝내자.", "웬만하면 오늘 끝내자."),
        ("몇일 뒤에 다시 연락하겠다.", "며칠 뒤에 다시 연락하겠다."),
    ];

    for (source, expected) in cases {
        let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());
        assert!(!diagnostics.is_empty(), "no diagnostic for {source:?}");
        assert_eq!(
            apply_safe_fixes(source, &diagnostics),
            expected,
            "{source:?}"
        );
    }
}

#[test]
fn keeps_contextual_barem_correction_review_only() {
    let config = LintConfig {
        profile: Profile::Strict,
        ..LintConfig::default()
    };
    let source = "작은 바램 하나를 적었다.";
    let diagnostics = lint_text(source, SourceKind::PlainText, &config);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.rule_id == "spelling.lexical.barem")
        .expect("바램 must be offered as a contextual review");

    assert_eq!(diagnostic.suggestions, ["바람"]);
    assert!(!diagnostic.safe_fix);
    assert_eq!(apply_safe_fixes(source, &diagnostics), source);
}

#[test]
fn catches_common_dependent_noun_spacing_variants() {
    let cases = [
        ("이 일은 할수 있다.", "이 일은 할 수 있다."),
        ("올것 같다.", "올 것 같다."),
        ("그 사람을 만난적 있다.", "그 사람을 만난 적 있다."),
        ("이유를 알수없다.", "이유를 알 수 없다."),
    ];

    for (source, expected) in cases {
        let diagnostics = lint_text(source, SourceKind::PlainText, &LintConfig::default());
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.original != diagnostic.suggestions[0]),
            "no actionable diagnostic for {source:?}: {diagnostics:?}"
        );
        let review = Engine::new(LintConfig::default())
            .check_with_fixes(source, SourceKind::PlainText, true)
            .review_fixed_text;
        assert_eq!(review, expected, "{source:?}");
    }
}

#[test]
fn catches_particle_errors_as_review_suggestions_without_touching_controls() {
    let config = LintConfig {
        profile: Profile::Strict,
        ..LintConfig::default()
    };
    let source = "연필를 샀고 친구은 웃었으며 동생와 의자을 옮겼다. 학교이 멀다.";
    let diagnostics = lint_text(source, SourceKind::PlainText, &config);

    for (original, suggestion) in [
        ("연필를", "연필을"),
        ("친구은", "친구는"),
        ("동생와", "동생과"),
        ("의자을", "의자를"),
        ("학교이", "학교가"),
    ] {
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.original == original
                    && diagnostic
                        .suggestions
                        .first()
                        .is_some_and(|value| value == suggestion)),
            "missing {original} -> {suggestion}: {diagnostics:?}"
        );
    }
    assert_eq!(apply_safe_fixes(source, &diagnostics), source);
}

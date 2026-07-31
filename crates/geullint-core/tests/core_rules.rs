use geullint_core::{
    Confidence, DictionaryOverlay, DocumentSession, Engine, FixSafety, LintConfig, MorphAnalyzer,
    Profile, RulePack, Severity, SourceKind, TextEdit, apply_safe_fixes, available_rule_ids,
    lint_text, rule_metadata,
};

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
            "grammar.particle.object-allomorph",
            "spacing.dependent-noun.jung",
            "punctuation.space-after-sentence-mark",
        ]
    );
    assert_eq!(diagnostics[0].suggestions, ["돼서"]);
    assert_eq!(diagnostics[2].suggestions, ["책을"]);
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

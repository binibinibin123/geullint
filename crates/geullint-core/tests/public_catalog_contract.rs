use geullint_core::{
    Confidence, LintConfig, Profile, SourceKind, available_rule_ids, lint_text, rule_metadata,
};

const MAX_RELEASE_RULE_COUNT: usize = 100;
const PUBLIC_CATEGORIES: &[&str] = &[
    "advanced",
    "grammar",
    "punctuation",
    "repetition",
    "spacing",
    "spelling",
    "style",
    "technical",
];

#[test]
fn ships_only_the_declared_curated_rule_core_with_metadata() {
    let rule_ids = available_rule_ids();
    let declared_count: usize = include_str!("../../../rules/catalog-count.txt")
        .trim()
        .parse()
        .expect("catalog-count.txt must contain an integer");

    assert!(
        declared_count <= MAX_RELEASE_RULE_COUNT,
        "the alpha release must not trade precision for an exact rule-count milestone"
    );
    assert_eq!(
        rule_ids.len(),
        declared_count,
        "the registry and the visible milestone must contain the same stable IDs"
    );

    for rule_id in rule_ids {
        let metadata = rule_metadata(&rule_id)
            .unwrap_or_else(|| panic!("{rule_id} must expose public metadata"));

        assert_eq!(metadata.id, rule_id);
        assert!(
            PUBLIC_CATEGORIES.contains(&metadata.category.as_str()),
            "{} has unsupported public category {:?}",
            metadata.id,
            metadata.category
        );
        assert!(
            metadata.documentation_url.ends_with(&format!("#{rule_id}")),
            "{} must link to its own documentation anchor",
            metadata.id
        );
    }
}

#[test]
fn excludes_unverified_context_free_spacing_and_casing_rules() {
    let rule_ids = available_rule_ids();

    for unverified in [
        "spacing.compound.gajokgwangye",
        "spacing.compound.pumjilgwanri",
        "spacing.compound.gogaekgwanri",
        "spacing.compound.jeongbogonggae",
        "technical.proper-name.python",
    ] {
        assert!(
            !rule_ids.contains(unverified),
            "{unverified} must remain outside the curated alpha catalogue"
        );
    }
}

#[test]
fn preserves_context_dependent_normal_phrases() {
    let config = LintConfig::default();

    for sentence in [
        "두 사람의 가족 관계가 복잡하다.",
        "제품의 품질 관리가 중요하다.",
        "고객 관리 업무를 맡았다.",
        "정보 공개를 청구했다.",
        "Python과 python이라는 문자열을 비교했다.",
    ] {
        let diagnostics = lint_text(sentence, SourceKind::PlainText, &config);
        assert!(
            diagnostics.is_empty(),
            "curated defaults must preserve {sentence:?}, got {diagnostics:?}"
        );
    }
}

#[test]
fn exposes_complete_bundled_metadata() {
    let metadata =
        rule_metadata("spelling.lexical.myeochil").expect("bundled rule metadata must exist");

    assert_eq!(metadata.title, "며칠 표기");
    assert_eq!(metadata.description, "‘몇일’을 표준어 ‘며칠’로 고칩니다.");
    assert_eq!(metadata.confidence, Confidence::High);
    assert!(metadata.default_enabled);
    assert!(
        metadata
            .incorrect_examples
            .iter()
            .any(|text| text == "몇일")
    );
    assert!(metadata.correct_examples.iter().any(|text| text == "며칠"));
}

#[test]
fn loads_category_catalogues_into_one_registry() {
    let rule_ids = available_rule_ids();

    assert!(rule_ids.contains("technical.term.web-browser"));
    assert!(rule_ids.contains("advanced.honorific.jeo-jasin"));
}

#[test]
fn every_public_rule_keeps_basic_matcher_contract_cases() {
    let config = LintConfig {
        profile: Profile::Editorial,
        ..LintConfig::default()
    };

    for rule_id in available_rule_ids() {
        let metadata =
            rule_metadata(&rule_id).unwrap_or_else(|| panic!("{rule_id} must expose metadata"));
        let incorrect = metadata
            .incorrect_examples
            .first()
            .unwrap_or_else(|| panic!("{rule_id} must have an incorrect example"));
        let correct = metadata
            .correct_examples
            .first()
            .unwrap_or_else(|| panic!("{rule_id} must have a correct example"));
        let positive_cases = [
            (SourceKind::PlainText, incorrect.clone()),
            (SourceKind::PlainText, format!("{incorrect}\n")),
            (SourceKind::PlainText, format!("{incorrect} 다음")),
            (SourceKind::Markdown, format!("{incorrect}\n\n")),
        ];
        let negative_cases = [
            (SourceKind::PlainText, correct.clone()),
            (SourceKind::PlainText, format!("{correct}\n")),
            (SourceKind::PlainText, format!("앞{correct}뒤")),
            (SourceKind::Markdown, format!("{correct}\n\n")),
        ];

        for (source_kind, source) in positive_cases {
            let diagnostics = lint_text(&source, source_kind, &config);
            assert!(
                diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.rule_id == rule_id),
                "{rule_id} must diagnose positive case {source:?} in {source_kind:?}"
            );
        }
        for (source_kind, source) in negative_cases {
            let diagnostics = lint_text(&source, source_kind, &config);
            assert!(
                diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.rule_id != rule_id),
                "{rule_id} must ignore negative case {source:?} in {source_kind:?}"
            );
        }
    }
}

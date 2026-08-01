use geullint_core::{
    Confidence, FixSafety, LintConfig, Profile, SourceKind, available_rule_ids, lint_text,
    rule_metadata,
};

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
fn every_public_rule_has_human_written_korean_metadata() {
    for rule_id in available_rule_ids() {
        let metadata =
            rule_metadata(&rule_id).unwrap_or_else(|| panic!("{rule_id} must expose metadata"));
        let has_hangul = |value: &str| {
            value
                .chars()
                .any(|character| ('가'..='힣').contains(&character))
        };

        assert!(
            has_hangul(&metadata.title),
            "{rule_id} has a placeholder title"
        );
        assert!(
            has_hangul(&metadata.description),
            "{rule_id} has a placeholder description"
        );
        assert!(
            !metadata.description.contains("한국어 검사 규칙입니다"),
            "{rule_id} has a generated placeholder description"
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
fn context_dependent_particle_and_jjigae_rules_are_non_default_review_rules() {
    for rule_id in [
        "grammar.particle.topic-allomorph",
        "grammar.particle.subject-allomorph",
        "grammar.particle.object-allomorph",
        "grammar.particle.comitative-allomorph",
        "grammar.particle.instrumental-allomorph",
        "spelling.lexical.jjigae",
    ] {
        let metadata =
            rule_metadata(rule_id).unwrap_or_else(|| panic!("{rule_id} must expose metadata"));

        assert!(
            !metadata.default_enabled,
            "{rule_id} must not be a default rule"
        );
        assert_eq!(metadata.fix_safety, FixSafety::Review, "{rule_id}");
        assert_eq!(metadata.profiles, [Profile::Strict, Profile::Editorial]);
    }
}

#[test]
fn ending_rule_metadata_matches_runtime_profile_and_fix_safety() {
    for (rule_id, profiles, safety, confidence) in [
        (
            "grammar.conjugation.doe-to-dwae",
            vec![Profile::Default, Profile::Strict, Profile::Editorial],
            FixSafety::Safe,
            Confidence::High,
        ),
        (
            "grammar.conjugation.dwae-to-doe",
            vec![Profile::Default, Profile::Strict, Profile::Editorial],
            FixSafety::Safe,
            Confidence::High,
        ),
        (
            "grammar.ending.euryeo",
            vec![Profile::Default, Profile::Strict, Profile::Editorial],
            FixSafety::Safe,
            Confidence::High,
        ),
        (
            "grammar.ending.euryeo-context",
            vec![Profile::Strict, Profile::Editorial],
            FixSafety::Review,
            Confidence::Medium,
        ),
        (
            "grammar.ending.colloquial-yong",
            vec![Profile::Editorial],
            FixSafety::Review,
            Confidence::Medium,
        ),
    ] {
        let metadata = rule_metadata(rule_id).unwrap_or_else(|| panic!("missing {rule_id}"));
        assert_eq!(metadata.profiles, profiles, "{rule_id}");
        assert_eq!(metadata.fix_safety, safety, "{rule_id}");
        assert_eq!(metadata.confidence, confidence, "{rule_id}");
        assert_eq!(
            metadata.default_enabled,
            profiles.contains(&Profile::Default)
        );
    }
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

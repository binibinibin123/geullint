#![cfg(feature = "standard")]

use geullint_core::{
    Engine, FixSafety, GeulRankSmall, LintConfig, Profile, SourceKind, StandardLexicon,
    StandardPipeline,
};

fn lexicon() -> StandardLexicon {
    StandardLexicon::parse(
        "geullint-standard-lexicon-v1\n감사해요\tEF\t1000\n며칠\tNNG\t8000\n문서\tNNG\t5000\n",
    )
    .expect("standard lexicon")
}

#[test]
fn standard_pipeline_merges_legacy_diagnostics_and_bounded_review_candidates() {
    let pipeline = StandardPipeline::new(
        Engine::new(LintConfig::default()),
        lexicon(),
        GeulRankSmall::default(),
    );

    let diagnostics = pipeline.check("문서 감사해용 몇일 뒤에 만나요.", SourceKind::PlainText);

    let oov = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.rule_id == "spelling.oov.near")
        .expect("bounded OOV diagnostic");
    assert!(!oov.suggestions.is_empty());
    assert!(oov.suggestions.len() <= 8);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_id == "spelling.lexical.myeochil")
    );
    assert!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_id == "spelling.oov.near")
            .all(|diagnostic| diagnostic.safety == FixSafety::Review)
    );
}

#[test]
fn standard_pipeline_generates_a_candidate_for_an_oov_hangul_word() {
    let pipeline = StandardPipeline::new(
        Engine::new(LintConfig::default()),
        StandardLexicon::parse("geullint-standard-lexicon-v1\n가다\tVV\t1000\n")
            .expect("standard lexicon"),
        GeulRankSmall::default(),
    );
    let diagnostics = pipeline.check("카츄", SourceKind::PlainText);
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.rule_id == "spelling.oov.near"
            && diagnostic.original == "카츄"
            && diagnostic
                .suggestions
                .iter()
                .any(|suggestion| suggestion.text == "가다")
    }));
}

#[test]
fn bundled_standard_pipeline_generates_an_oov_candidate() {
    let pipeline = StandardPipeline::new(
        Engine::new(LintConfig::default()),
        StandardLexicon::bundled().expect("bundled standard lexicon"),
        GeulRankSmall::bundled().expect("bundled ranker"),
    );
    let diagnostics = pipeline.check("카츄", SourceKind::PlainText);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_id == "spelling.oov.near")
    );
}

#[test]
fn standard_review_preview_does_not_apply_a_two_edit_distance_oov_guess() {
    let pipeline = StandardPipeline::new(
        Engine::new(LintConfig::default()),
        StandardLexicon::parse("geullint-standard-lexicon-v1\n가다\tVV\t1000\n")
            .expect("standard lexicon"),
        GeulRankSmall::default(),
    );
    let outcome = pipeline.check_with_fixes("카츄", SourceKind::PlainText, true);
    assert_eq!(outcome.review_fixed_text, "카츄");
}

#[test]
fn standard_pipeline_does_not_apply_unvalidated_candidates_to_fixed_text() {
    let pipeline = StandardPipeline::new(
        Engine::new(LintConfig {
            profile: Profile::Default,
            ..LintConfig::default()
        }),
        lexicon(),
        GeulRankSmall::default(),
    );

    let outcome = pipeline.check_with_fixes(
        "문서 감사해용 몇일 뒤에 만나요.",
        SourceKind::PlainText,
        true,
    );

    assert!(
        outcome
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_id == "spelling.oov.near")
    );
    assert!(outcome.fixed_text.contains("감사해용"));
    assert!(outcome.fixed_text.contains("며칠"));
}

#[test]
fn standard_pipeline_keeps_unvalidated_review_candidates_out_of_preview_text() {
    let pipeline = StandardPipeline::new(
        Engine::new(LintConfig::default()),
        lexicon(),
        GeulRankSmall::default(),
    );
    let text = "문서 감사해요 몇일 뒤에 만나요.";
    let candidate = pipeline
        .check(text, SourceKind::PlainText)
        .into_iter()
        .find(|diagnostic| diagnostic.rule_id == "spelling.oov.near")
        .expect("review candidate");
    assert!(!candidate.suggestions.is_empty());

    let without_review = pipeline.check_with_fixes(text, SourceKind::PlainText, false);
    assert_eq!(without_review.review_fixed_text, without_review.fixed_text);

    let with_review = pipeline.check_with_fixes(text, SourceKind::PlainText, true);
    assert_eq!(with_review.review_fixed_text, with_review.fixed_text);
    assert!(
        with_review
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_id == "spelling.oov.near")
    );
}

#[test]
fn standard_pipeline_can_opt_into_the_learned_context_ranker_without_safe_fixes() {
    let pipeline = StandardPipeline::bundled_with_context(LintConfig::default())
        .expect("bundled context ranker");
    assert!(pipeline.has_context_ranker());
    let outcome = pipeline.check_with_fixes("문새를 저장합니다.", SourceKind::PlainText, true);
    assert!(outcome.diagnostics.iter().all(|diagnostic| {
        diagnostic
            .suggestions
            .iter()
            .all(|suggestion| suggestion.safety == FixSafety::Review)
    }));
    assert_eq!(outcome.fixed_text, "문새를 저장합니다.");
}

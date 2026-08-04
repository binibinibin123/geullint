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

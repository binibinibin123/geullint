use geullint_core::{
    AnalyzedDocument, Candidate, Confidence, Evidence, GeulRankSmall, LintConfig, RuleContext,
    SourceKind, TextRange,
};

fn context() -> RuleContext<'static> {
    let text: &'static str = Box::leak("몇일".to_owned().into_boxed_str());
    let document: &'static AnalyzedDocument =
        Box::leak(Box::new(AnalyzedDocument::new(text, SourceKind::PlainText)));
    let config: &'static LintConfig = Box::leak(Box::new(LintConfig::default()));
    RuleContext::new(text, SourceKind::PlainText, document, config)
}

#[test]
fn local_ranker_prefers_lower_edit_distance_and_higher_frequency() {
    let ranker = GeulRankSmall::default();
    let context = context();
    let near = Candidate::new(
        "spelling.oov.near",
        TextRange { start: 0, end: 6 },
        "몇일",
        "며칠",
    )
    .with_evidence(Evidence::new("edit-distance", "2", 0.5))
    .with_evidence(Evidence::new("phonology-distance", "2", 0.5))
    .with_evidence(Evidence::new("frequency", "10000", 0.8));
    let far = Candidate::new(
        "spelling.oov.near",
        TextRange { start: 0, end: 6 },
        "몇일",
        "가다",
    )
    .with_evidence(Evidence::new("edit-distance", "2", 0.5))
    .with_evidence(Evidence::new("phonology-distance", "6", 0.2))
    .with_evidence(Evidence::new("frequency", "100", 0.2));

    assert!(ranker.score(&near, &context) > ranker.score(&far, &context));
    let mut candidates = vec![far, near];
    ranker.rank(&mut candidates, &context);
    assert_eq!(candidates[0].replacement, "며칠");
    assert_eq!(ranker.confidence(candidates[0].score), Confidence::High);
}

#[test]
#[allow(clippy::float_cmp)]
fn ranker_is_bounded_and_deterministic_for_empty_evidence() {
    let ranker = GeulRankSmall::default();
    let context = context();
    let candidate = Candidate::new("unknown", TextRange { start: 0, end: 6 }, "몇일", "며칠");
    let first = ranker.score(&candidate, &context);
    let second = ranker.score(&candidate, &context);
    assert_eq!(first, second);
    assert!((0.0..=1.0).contains(&first));
}

#[cfg(feature = "standard")]
#[test]
fn bundled_ranker_artifact_round_trips_to_the_runtime_contract() {
    let ranker = GeulRankSmall::bundled().expect("bundled ranker");
    assert!((ranker.weights().bias - 4.0).abs() < 0.05);
}

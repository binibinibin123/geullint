use geullint_core::{
    AnalyzedDocument, CandidateGenerator, LintConfig, RuleContext, SourceKind,
    SpacingCandidateGenerator, SpellingCandidateGenerator, StandardLexicon, TextRange,
};

fn lexicon() -> StandardLexicon {
    StandardLexicon::parse(
        "geullint-standard-lexicon-v1\n가다\tVV\t100\n며칠\tNNG\t10000\n문서\tNNG\t1000\n수\tNNG\t5000\n읽다\tVV\t3000\n할\tVV\t7000\n",
    )
    .expect("test lexicon")
}

#[test]
fn spelling_generator_ranks_a_phonologically_close_standard_word() {
    let text = "몇일 뒤";
    let document = AnalyzedDocument::new(text, SourceKind::PlainText);
    let config = LintConfig::default();
    let context = RuleContext::new(text, SourceKind::PlainText, &document, &config);
    let generator = SpellingCandidateGenerator::new(lexicon(), 32);
    let candidates = generator.generate(&context);

    assert_eq!(
        candidates
            .first()
            .map(|candidate| candidate.replacement.as_str()),
        Some("며칠")
    );
    assert_eq!(
        candidates.first().map(|candidate| candidate.range),
        Some(TextRange { start: 0, end: 6 })
    );
    assert!(candidates.len() <= 32);
}

#[test]
fn spelling_generator_is_quiet_for_known_words_and_short_tokens() {
    let text = "며칠 가";
    let document = AnalyzedDocument::new(text, SourceKind::PlainText);
    let config = LintConfig::default();
    let context = RuleContext::new(text, SourceKind::PlainText, &document, &config);
    let candidates = SpellingCandidateGenerator::new(lexicon(), 32).generate(&context);
    assert!(candidates.is_empty());
}

#[test]
fn spacing_generator_proposes_only_lexicon_backed_join_and_split_candidates() {
    let text = "할수 문서 읽다";
    let document = AnalyzedDocument::new(text, SourceKind::PlainText);
    let config = LintConfig::default();
    let context = RuleContext::new(text, SourceKind::PlainText, &document, &config);
    let candidates = SpacingCandidateGenerator::new(lexicon(), 32).generate(&context);

    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.replacement == "할 수")
    );
    assert!(
        candidates
            .iter()
            .all(|candidate| candidate.range.end <= text.len())
    );
    assert!(candidates.len() <= 32);
}

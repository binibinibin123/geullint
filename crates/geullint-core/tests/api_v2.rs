use geullint_core::{
    AnalyzedDocument, Candidate, Confidence, Diagnostic, DiagnosticV2, Engine, Evidence, FixPolicy,
    FixSafety, LintConfig, Pipeline, Profile, RuleContext, Severity, SourceKind, TextRange,
};

#[test]
fn v2_diagnostic_preserves_the_legacy_contract_and_adds_evidence() {
    let legacy = Diagnostic {
        rule_id: "spelling.lexical.myeochil".to_owned(),
        severity: Severity::Error,
        message: "표준어는 며칠입니다.".to_owned(),
        range: TextRange { start: 0, end: 6 },
        original: "몇일".to_owned(),
        suggestions: vec!["며칠".to_owned()],
        safe_fix: true,
    };
    let converted = DiagnosticV2::from_legacy(&legacy);

    assert_eq!(converted.rule_id, legacy.rule_id);
    assert_eq!(converted.range, legacy.range);
    assert_eq!(converted.suggestions[0].text, "며칠");
    assert_eq!(converted.safety, FixSafety::Safe);
    assert_eq!(converted.evidence[0].code, "legacy-rule");
}

#[test]
fn compatibility_pipeline_keeps_order_ranges_and_fixed_text() {
    let engine = Engine::new(LintConfig {
        profile: Profile::Default,
        ..LintConfig::default()
    });
    let legacy = engine.check_with_fixes("몇일 뒤에 만나요.", SourceKind::PlainText, false);
    let v2 =
        Pipeline::new(&engine).check_with_fixes("몇일 뒤에 만나요.", SourceKind::PlainText, false);

    assert_eq!(v2.fixed_text, legacy.fixed_text);
    assert_eq!(v2.review_fixed_text, legacy.review_fixed_text);
    assert_eq!(
        v2.diagnostics
            .iter()
            .map(|item| (item.rule_id.as_str(), item.range))
            .collect::<Vec<_>>(),
        legacy
            .diagnostics
            .iter()
            .map(|item| (item.rule_id.as_str(), item.range))
            .collect::<Vec<_>>(),
    );
}

#[test]
fn context_and_candidate_carry_source_safe_ranges() {
    let text = "몇일 뒤에";
    let document = AnalyzedDocument::new(text, SourceKind::PlainText);
    let config = LintConfig::default();
    let context = RuleContext::new(text, SourceKind::PlainText, &document, &config);
    let candidate = Candidate::new(
        "spelling.lexical.myeochil",
        TextRange { start: 0, end: 6 },
        "몇일",
        "며칠",
    );

    assert_eq!(context.source_kind(), SourceKind::PlainText);
    assert_eq!(context.text(), text);
    assert_eq!(candidate.original, "몇일");
    assert!(candidate.range.end <= text.len());
}

#[test]
fn fix_policy_maps_safe_review_and_abstain_explicitly() {
    assert_eq!(
        FixPolicy::from_safety(true, Confidence::High),
        FixPolicy::Safe
    );
    assert_eq!(
        FixPolicy::from_safety(true, Confidence::Low),
        FixPolicy::Review
    );
    assert_eq!(
        FixPolicy::from_safety(false, Confidence::High),
        FixPolicy::Abstain
    );
}

#[test]
fn evidence_is_serializable_without_exposing_source_text() {
    let evidence = Evidence::new("morphology", "NNG+JX", 0.82);
    let json = serde_json::to_value(&evidence).expect("evidence serializes");
    assert_eq!(json["code"], "morphology");
    assert_eq!(json["value"], "NNG+JX");
    assert_eq!(json["score"], 0.82);
    assert!(!json.as_object().expect("object").contains_key("text"));
}

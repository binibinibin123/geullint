use geullint_core::{Confidence, FixPolicy, PolicyDecision, PolicyThresholds, StyleProfile};

#[test]
fn calibrated_policy_only_marks_high_confidence_objective_fixes_safe() {
    let thresholds = PolicyThresholds::default();
    let safe = thresholds.decide(
        "spelling.lexical",
        true,
        Confidence::High,
        0.995,
        StyleProfile::Plain,
    );
    let review = thresholds.decide(
        "proper-noun",
        true,
        Confidence::High,
        0.999,
        StyleProfile::Plain,
    );
    let abstain = thresholds.decide("unknown", false, Confidence::Low, 0.2, StyleProfile::Plain);

    assert_eq!(safe.policy, FixPolicy::Safe);
    assert_eq!(review.policy, FixPolicy::Review);
    assert_eq!(abstain.policy, FixPolicy::Abstain);
    assert!(safe.reason.contains("calibrated"));
}

#[test]
fn style_and_score_dead_bands_prevent_overconfident_editorial_fixes() {
    let thresholds = PolicyThresholds::default();
    let low_score = thresholds.decide(
        "spelling.lexical",
        true,
        Confidence::High,
        0.7,
        StyleProfile::Plain,
    );
    let editorial = thresholds.decide(
        "style.concise",
        true,
        Confidence::High,
        0.99,
        StyleProfile::Formal,
    );

    assert_eq!(low_score.policy, FixPolicy::Review);
    assert_eq!(editorial.policy, FixPolicy::Review);
}

#[test]
fn policy_decision_is_serializable_and_exposes_no_input_text() {
    let decision = PolicyDecision::new(
        FixPolicy::Review,
        Confidence::Medium,
        "margin below safe threshold",
    );
    let json = serde_json::to_value(&decision).expect("policy decision serializes");
    assert_eq!(json["policy"], "review");
    assert!(!json.as_object().expect("object").contains_key("text"));
}

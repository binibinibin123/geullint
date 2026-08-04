#![cfg(feature = "standard")]

use geullint_wasm::{WasmLintRequest, evaluate_context, evaluate_standard};

#[test]
fn standard_wasm_entry_point_exposes_review_candidates_without_changing_safe_text() {
    let request =
        WasmLintRequest::from_json(r#"{"text":"문서느 검사", "sourceKind":"plain_text"}"#)
            .expect("valid browser request");
    let response = evaluate_standard(&request);

    assert!(
        response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_id == "spelling.oov.near")
    );
    assert!(
        response
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_id == "spelling.oov.near")
            .all(|diagnostic| diagnostic.safety == geullint_core::FixSafety::Review)
    );
    assert!(response.fixed_text.contains("문서느"));
}

#[test]
fn context_wasm_entry_point_is_opt_in_and_keeps_candidates_review_only() {
    let request =
        WasmLintRequest::from_json(r#"{"text":"문새를 저장합니다.","sourceKind":"plain_text"}"#)
            .expect("valid browser request");
    let response = evaluate_context(&request);

    let candidate = response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.rule_id == "spelling.oov.near")
        .expect("learned candidate diagnostic");
    assert!(
        candidate
            .suggestions
            .iter()
            .all(|suggestion| { suggestion.safety == geullint_core::FixSafety::Review })
    );
    assert_eq!(response.fixed_text, "문새를 저장합니다.");
}

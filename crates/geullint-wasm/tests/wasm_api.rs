use geullint_wasm::{WasmLintRequest, evaluate, lint_json, rule_catalog_json};

#[test]
fn returns_the_same_offline_diagnostic_contract_as_the_native_core() {
    let request = WasmLintRequest::from_json(
        r#"{"text":"몇일 뒤에 만나요.","sourceKind":"plain_text","config":{"profile":"default"}}"#,
    )
    .expect("valid browser lint request");
    let response = evaluate(&request);

    assert_eq!(response.version, 1);
    assert_eq!(response.diagnostics.len(), 1);
    assert_eq!(response.diagnostics[0].rule_id, "spelling.lexical.myeochil");
}

#[test]
fn returns_the_complete_safely_corrected_text() {
    let request = WasmLintRequest::from_json(
        r#"{"text":"몇일 문서를 문서를 저장합니다.","sourceKind":"plain_text"}"#,
    )
    .expect("valid browser lint request");
    let response = evaluate(&request);

    assert_eq!(response.fixed_text, "며칠 문서를 문서를 저장합니다.");
    assert!(response.diagnostics.iter().any(|item| !item.safe_fix));
}

#[test]
fn serializes_the_corrected_text_for_the_browser_worker() {
    let response: serde_json::Value = serde_json::from_str(
        &lint_json(r#"{"text":"몇일입니다.","sourceKind":"plain_text"}"#)
            .expect("browser response JSON"),
    )
    .expect("valid browser response");

    assert_eq!(response["fixedText"], "며칠입니다.");
    assert!(response.get("fixed_text").is_none());
}

#[test]
fn carries_the_editorial_profile_through_the_wasm_boundary() {
    let request = WasmLintRequest::from_json(
        r#"{"text":"그것은 가장 최고다.","sourceKind":"plain_text","config":{"profile":"editorial"}}"#,
    )
    .expect("valid browser lint request");
    let response = evaluate(&request);

    assert_eq!(
        response.diagnostics[0].rule_id,
        "style.redundancy.gajang-choego"
    );
}

#[test]
fn carries_dictionary_overlay_terms_through_the_wasm_boundary() {
    let request = WasmLintRequest::from_json(
        r#"{"text":"몇일 뒤에 만나요.","sourceKind":"plain_text","config":{"dictionaryOverlay":["몇일"]}}"#,
    )
    .expect("valid browser lint request");
    let response = evaluate(&request);

    assert!(response.diagnostics.is_empty());
}

#[test]
fn scans_browser_code_comments_without_scanning_string_literals() {
    let request = WasmLintRequest::from_json(
        r#"{"text":"const label = '몇일'; // 몇일","sourceKind":"javascript"}"#,
    )
    .expect("valid browser lint request");
    let response = evaluate(&request);

    assert_eq!(response.diagnostics.len(), 1);
    assert_eq!(response.diagnostics[0].original, "몇일");
}

#[test]
fn exposes_the_same_ordered_curated_rule_catalogue_to_the_browser() {
    let catalog: serde_json::Value =
        serde_json::from_str(&rule_catalog_json().expect("catalogue JSON"))
            .expect("valid catalogue JSON");
    let rules = catalog["rules"].as_array().expect("rules array");

    assert_eq!(catalog["version"], 1);
    let declared_count: usize = include_str!("../../../rules/catalog-count.txt")
        .trim()
        .parse()
        .expect("catalog-count.txt must contain an integer");
    assert!(declared_count <= 100);
    assert_eq!(catalog["ruleCount"], declared_count);
    assert_eq!(rules.len(), declared_count);
    assert!(
        rules
            .windows(2)
            .all(|pair| pair[0]["id"].as_str() < pair[1]["id"].as_str())
    );
}

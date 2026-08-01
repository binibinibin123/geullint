use geullint_core::{Engine, LintConfig, Profile, SourceKind, lint_text};
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
fn user_sentence_exposes_safe_and_review_corrections() {
    let request = WasmLintRequest::from_json(
        r#"{"text":"안녕하세용 감사해용 왠만하면 돼게 할려고 하였다","sourceKind":"plain_text","config":{"profile":"editorial"}}"#,
    )
    .expect("valid browser lint request");
    let response = evaluate(&request);

    assert_eq!(
        response
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.rule_id.as_str())
            .collect::<Vec<_>>(),
        [
            "grammar.ending.colloquial-yong",
            "grammar.ending.colloquial-yong",
            "spelling.confusable.wen-waen",
            "grammar.conjugation.dwae-to-doe",
            "grammar.ending.euryeo",
        ]
    );
    assert_eq!(
        response.fixed_text,
        "안녕하세용 감사해용 웬만하면 되게 하려고 하였다"
    );
    assert_eq!(
        response
            .diagnostics
            .iter()
            .filter(|item| item.safe_fix)
            .count(),
        3
    );
}

#[test]
fn productive_endings_match_native_core_diagnostics_and_fixed_text() {
    let text = "😀 안돼게요 재확인할려고도 됀다면 갈려고 감사해용 읽읍니다 확인하십시요 아니예요 않됩니다 묵을데가 입은채로 모르는듯하다 먹을만큼 들은대로 사는법이다 잊을리가 없다";

    for profile in [Profile::Default, Profile::Strict, Profile::Editorial] {
        let config = LintConfig {
            profile,
            ..LintConfig::default()
        };
        let native = lint_text(text, SourceKind::PlainText, &config);
        let request_json = serde_json::json!({
            "text": text,
            "sourceKind": "plain_text",
            "config": { "profile": profile }
        })
        .to_string();
        let request = WasmLintRequest::from_json(&request_json).expect("valid browser request");
        let browser = evaluate(&request);

        assert_eq!(browser.diagnostics, native, "{profile:?}");
        assert_eq!(
            browser.fixed_text,
            Engine::new(config).fix(text, SourceKind::PlainText),
            "{profile:?}"
        );
    }
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
fn browser_fix_resolves_chained_safe_rules_in_one_request() {
    let response: serde_json::Value = serde_json::from_str(
        &lint_json(r#"{"text":"사과,,,,배와 않돼게 처리된 됬읍니다.","sourceKind":"plain_text"}"#)
            .expect("browser response JSON"),
    )
    .expect("valid browser response");

    assert_eq!(response["fixedText"], "사과, 배와 안 되게 처리된 됐습니다.");
}

#[test]
fn browser_returns_a_stable_review_preview_separately_from_safe_fixes() {
    let response: serde_json::Value = serde_json::from_str(
        &lint_json(
            r#"{"text":"사과,,,,배를 샀어요 감사해용","sourceKind":"plain_text","config":{"profile":"editorial"}}"#,
        )
        .expect("browser response JSON"),
    )
    .expect("valid browser response");

    assert_eq!(response["fixedText"], "사과, 배를 샀어요 감사해용");
    assert_eq!(response["reviewFixedText"], "사과, 배를 샀어요 감사해요");
}

#[test]
fn browser_can_skip_review_preview_work_without_hiding_review_diagnostics() {
    let response: serde_json::Value = serde_json::from_str(
        &lint_json(
            r#"{"text":"사과,,,,배를 샀어요 감사해용","sourceKind":"plain_text","config":{"profile":"editorial"},"includeReviewFixes":false}"#,
        )
        .expect("browser response JSON"),
    )
    .expect("valid browser response");

    assert_eq!(response["fixedText"], "사과, 배를 샀어요 감사해용");
    assert_eq!(response["reviewFixedText"], response["fixedText"]);
    assert!(
        response["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|diagnostic| diagnostic["safeFix"] == false)
    );
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
    assert_eq!(catalog["ruleCount"], declared_count);
    assert_eq!(rules.len(), declared_count);
    assert!(
        rules
            .windows(2)
            .all(|pair| pair[0]["id"].as_str() < pair[1]["id"].as_str())
    );
}

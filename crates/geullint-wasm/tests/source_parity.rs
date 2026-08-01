use geullint_core::{LintConfig, Profile, SourceKind};
use geullint_wasm::{WasmLintRequest, evaluate};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Fixture {
    version: u8,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: String,
    source_kind: SourceKind,
    profile: Profile,
    text: String,
    diagnostics: Vec<ExpectedDiagnostic>,
    fixed_text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedDiagnostic {
    rule_id: String,
    start: usize,
    end: usize,
    original: String,
    suggestion: String,
    safe_fix: bool,
}

#[test]
fn wasm_host_matches_the_shared_source_contract() {
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../geullint-core/tests/fixtures/source-parity.json"
    ))
    .expect("source parity fixture must be valid JSON");
    assert_eq!(fixture.version, 1);

    for case in fixture.cases {
        let response = evaluate(&WasmLintRequest {
            text: case.text,
            source_kind: case.source_kind,
            config: LintConfig {
                profile: case.profile,
                ..LintConfig::default()
            },
            include_review_fixes: false,
        });

        assert_eq!(
            response.diagnostics.len(),
            case.diagnostics.len(),
            "{}",
            case.id
        );
        for (actual, expected) in response.diagnostics.iter().zip(&case.diagnostics) {
            assert_eq!(actual.rule_id, expected.rule_id, "{}", case.id);
            assert_eq!(actual.range.start, expected.start, "{}", case.id);
            assert_eq!(actual.range.end, expected.end, "{}", case.id);
            assert_eq!(actual.original, expected.original, "{}", case.id);
            assert_eq!(
                actual.suggestions.first(),
                Some(&expected.suggestion),
                "{}",
                case.id
            );
            assert_eq!(actual.safe_fix, expected.safe_fix, "{}", case.id);
        }
        assert_eq!(response.fixed_text, case.fixed_text, "{}", case.id);
    }
}

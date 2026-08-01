use geullint_core::{Engine, LintConfig, Profile, SourceKind, lint_text};
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

fn fixture() -> Fixture {
    serde_json::from_str(include_str!("fixtures/source-parity.json"))
        .expect("source parity fixture must be valid JSON")
}

#[test]
fn source_fixture_has_a_stable_version_and_unique_case_ids() {
    let fixture = fixture();
    assert_eq!(fixture.version, 1);

    let mut ids = fixture
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), fixture.cases.len());
}

#[test]
fn source_fixture_matches_the_complete_ordered_native_contract() {
    for case in fixture().cases {
        let config = LintConfig {
            profile: case.profile,
            ..LintConfig::default()
        };
        let diagnostics = lint_text(&case.text, case.source_kind, &config);

        assert_eq!(diagnostics.len(), case.diagnostics.len(), "{}", case.id);
        for (actual, expected) in diagnostics.iter().zip(&case.diagnostics) {
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
        assert_eq!(
            Engine::new(config).fix(&case.text, case.source_kind),
            case.fixed_text,
            "{}",
            case.id
        );
    }
}

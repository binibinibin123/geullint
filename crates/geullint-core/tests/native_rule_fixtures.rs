use geullint_core::{LintConfig, Profile, SourceKind, lint_text};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

const DYNAMIC_NATIVE_RULE_IDS: &[&str] = &[
    "grammar.ending.deun-choice",
    "grammar.particle.topic-allomorph",
    "grammar.particle.subject-allomorph",
    "grammar.particle.object-allomorph",
    "grammar.particle.comitative-allomorph",
    "grammar.particle.instrumental-allomorph",
    "punctuation.space-after-comma",
    "punctuation.space-after-sentence-mark",
    "repetition.adjacent-word",
];

#[derive(Debug, Deserialize)]
struct FixtureFile {
    version: u8,
    cases: Vec<FixtureCase>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FixtureCase {
    name: String,
    kind: FixtureKind,
    text: String,
    source_kind: SourceKind,
    profile: Profile,
    expected: Vec<ExpectedDiagnostic>,
    #[serde(default)]
    absent_rule_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
enum FixtureKind {
    Error,
    Normal,
    Exception,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedDiagnostic {
    rule_id: String,
    #[serde(default = "one")]
    count: usize,
    #[serde(default)]
    suggestions: Vec<String>,
}

const fn one() -> usize {
    1
}

#[test]
#[allow(clippy::too_many_lines)] // Keeps the fixture schema contract in one readable assertion flow.
fn native_rule_fixtures_cover_each_dynamic_rule_and_its_known_good_forms() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("native-rules.yaml");
    let fixture_file: FixtureFile = serde_yaml::from_str(
        &fs::read_to_string(&fixture_path)
            .unwrap_or_else(|error| panic!("{}: {error}", fixture_path.display())),
    )
    .expect("native rule fixture YAML must be valid");
    assert_eq!(
        fixture_file.version, 1,
        "fixture schema version must be explicit"
    );

    let mut covered_rule_ids = BTreeSet::new();
    let mut quiet_rule_ids = BTreeSet::new();
    for case in fixture_file.cases {
        let diagnostics = lint_text(
            &case.text,
            case.source_kind,
            &LintConfig {
                profile: case.profile,
                ..LintConfig::default()
            },
        );

        let mut expected_rule_ids = Vec::new();
        for expected in case.expected {
            covered_rule_ids.insert(expected.rule_id.clone());
            expected_rule_ids.extend(std::iter::repeat_n(
                expected.rule_id.clone(),
                expected.count,
            ));
            let matching: Vec<_> = diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.rule_id == expected.rule_id)
                .collect();
            assert_eq!(
                matching.len(),
                expected.count,
                "{}: expected {} occurrence(s) of {}",
                case.name,
                expected.count,
                expected.rule_id,
            );
            if !expected.suggestions.is_empty() {
                assert_eq!(
                    matching
                        .iter()
                        .map(|diagnostic| diagnostic.suggestions.clone())
                        .collect::<Vec<_>>(),
                    expected
                        .suggestions
                        .iter()
                        .map(|suggestion| vec![suggestion.clone()])
                        .collect::<Vec<_>>(),
                    "{}: unexpected suggestion for {}",
                    case.name,
                    expected.rule_id,
                );
            }
        }

        for absent_rule_id in case.absent_rule_ids {
            if case.kind != FixtureKind::Error {
                quiet_rule_ids.insert(absent_rule_id.clone());
            }
            assert!(
                diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.rule_id != absent_rule_id),
                "{}: {} must not report {}",
                case.name,
                case.text,
                absent_rule_id,
            );
        }
        expected_rule_ids.sort_unstable();
        let mut actual_rule_ids: Vec<_> = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.rule_id.clone())
            .collect();
        actual_rule_ids.sort_unstable();
        assert_eq!(
            actual_rule_ids, expected_rule_ids,
            "{}: the fixture must not permit an unexpected diagnostic",
            case.name,
        );
    }

    assert_eq!(
        covered_rule_ids,
        DYNAMIC_NATIVE_RULE_IDS
            .iter()
            .map(|rule_id| (*rule_id).to_owned())
            .collect(),
        "every dynamic native rule requires an executable positive fixture",
    );
    assert_eq!(
        quiet_rule_ids,
        DYNAMIC_NATIVE_RULE_IDS
            .iter()
            .map(|rule_id| (*rule_id).to_owned())
            .collect(),
        "every dynamic native rule requires a normal or exception fixture",
    );
}

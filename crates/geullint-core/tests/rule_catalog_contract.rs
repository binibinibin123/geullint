use geullint_core::{LintConfig, Profile, SourceKind, apply_safe_fixes, lint_text};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Catalog {
    rules: Vec<CatalogRule>,
}

#[derive(Debug, Deserialize)]
struct CatalogRule {
    id: String,
    #[serde(rename = "safeFix")]
    safe_fix: bool,
    replacements: Vec<Replacement>,
}

#[test]
fn every_safe_catalogue_fix_is_idempotent_and_removes_its_own_diagnostic() {
    let catalog: Catalog = serde_yaml::from_str(include_str!("../../../rules/ko-basic.yaml"))
        .expect("bundled rule catalogue must be valid YAML");
    let config = LintConfig {
        profile: Profile::Editorial,
        ..LintConfig::default()
    };

    for rule in catalog.rules.into_iter().filter(|rule| rule.safe_fix) {
        for replacement in rule.replacements {
            let diagnostics = lint_text(&replacement.from, SourceKind::PlainText, &config);
            let fixed = apply_safe_fixes(&replacement.from, &diagnostics);
            assert_eq!(
                fixed, replacement.to,
                "{} must apply its exact safe fix",
                rule.id
            );

            let after_first_fix = lint_text(&fixed, SourceKind::PlainText, &config);
            let after_second_fix = apply_safe_fixes(&fixed, &after_first_fix);
            assert_eq!(fixed, after_second_fix, "{} must be idempotent", rule.id);
            assert!(
                after_first_fix
                    .iter()
                    .all(|diagnostic| diagnostic.rule_id != rule.id),
                "{} must remove its own diagnostic after a safe fix",
                rule.id,
            );
        }
    }
}

#[derive(Debug, Deserialize)]
struct Replacement {
    from: String,
    to: String,
}

#[test]
fn every_catalogued_replacement_has_a_detection_and_a_non_recursive_fix_case() {
    let catalog: Catalog = serde_yaml::from_str(include_str!("../../../rules/ko-basic.yaml"))
        .expect("bundled rule catalogue must be valid YAML");
    let config = LintConfig {
        profile: Profile::Editorial,
        ..LintConfig::default()
    };

    for rule in catalog.rules {
        for replacement in rule.replacements {
            let detected = lint_text(&replacement.from, SourceKind::PlainText, &config);
            assert!(
                detected
                    .iter()
                    .any(|diagnostic| diagnostic.rule_id == rule.id),
                "{} must detect {:?}",
                rule.id,
                replacement.from,
            );

            let corrected = lint_text(&replacement.to, SourceKind::PlainText, &config);
            assert!(
                corrected
                    .iter()
                    .all(|diagnostic| diagnostic.rule_id != rule.id),
                "{} must not diagnose its own suggested correction {:?}",
                rule.id,
                replacement.to,
            );
        }
    }
}

#[test]
fn every_catalogued_replacement_is_detected_in_prose_and_supported_code_comments() {
    let catalog: Catalog = serde_yaml::from_str(include_str!("../../../rules/ko-basic.yaml"))
        .expect("bundled rule catalogue must be valid YAML");
    let config = LintConfig {
        profile: Profile::Editorial,
        ..LintConfig::default()
    };

    for rule in catalog.rules {
        for replacement in rule.replacements {
            let cases = [
                (
                    SourceKind::Markdown,
                    format!("본문에 {} 오류가 있습니다.", replacement.from),
                ),
                (
                    SourceKind::JavaScript,
                    format!("const label = \"정상 문자열\"; // {}", replacement.from),
                ),
            ];
            for (source_kind, source) in cases {
                let diagnostics = lint_text(&source, source_kind, &config);
                assert!(
                    diagnostics
                        .iter()
                        .any(|diagnostic| diagnostic.rule_id == rule.id),
                    "{} must detect {:?} in {source_kind:?}",
                    rule.id,
                    replacement.from,
                );
            }
        }
    }
}

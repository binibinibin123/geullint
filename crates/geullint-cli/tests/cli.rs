use assert_cmd::Command;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

#[test]
fn reports_a_fix_and_exits_one_when_a_file_has_an_error() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("memo.txt");
    fs::write(&input, "몇일 뒤에 만나요.").expect("test input");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command.arg(&input);
    command
        .assert()
        .code(1)
        .stdout(predicates::str::contains("spelling.lexical.myeochil"))
        .stdout(predicates::str::contains("며칠"));
}

#[test]
fn emits_machine_readable_json_when_requested() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("memo.md");
    fs::write(&input, "금새 알려 드릴게요.").expect("test input");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--format", "json"])
        .arg(&input)
        .output()
        .expect("run geullint");

    assert_eq!(output.status.code(), Some(1));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(
        document["diagnostics"][0]["ruleId"],
        "spelling.lexical.geumse"
    );
    assert_eq!(document["diagnostics"][0]["suggestions"][0], "금세");
}

#[test]
fn directory_scan_skips_binary_and_invalid_utf8_files() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let prose = directory.path().join("memo.md");
    let binary = directory.path().join("preview.gif");
    let invalid_text = directory.path().join("broken.txt");
    fs::write(&prose, "몇일 뒤에 만나요.").expect("prose input");
    fs::write(&binary, [0xff, 0xd8, 0xff, 0x00]).expect("binary input");
    fs::write(&invalid_text, [0xff, 0xfe, 0x00]).expect("invalid text input");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--format", "json"])
        .arg(directory.path())
        .output()
        .expect("run geullint");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid JSON output");
    let diagnostics = document["diagnostics"]
        .as_array()
        .expect("diagnostic array");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["path"], prose.display().to_string());
    assert_eq!(diagnostics[0]["ruleId"], "spelling.lexical.myeochil");
}

#[test]
fn explicitly_selected_invalid_utf8_file_remains_an_error() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("broken.txt");
    fs::write(&input, [0xff, 0xfe, 0x00]).expect("invalid text input");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command
        .arg(&input)
        .assert()
        .code(2)
        .stderr(predicates::str::contains("UTF-8"));
}

#[test]
fn directory_fix_skips_unsupported_structured_text_formats() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let prose = directory.path().join("memo.txt");
    let mdx = directory.path().join("page.mdx");
    let restructured_text = directory.path().join("guide.rst");
    let json = directory.path().join("data.json");
    let mdx_source = "export const slug = \"몇일\";";
    let rst_source = ".. code-block:: python\n\n    value = \"몇일\"";
    let json_source = r#"{"메세지":"몇일"}"#;
    fs::write(&prose, "몇일 뒤에 만나요.").expect("plain text input");
    fs::write(&mdx, mdx_source).expect("MDX input");
    fs::write(&restructured_text, rst_source).expect("reStructuredText input");
    fs::write(&json, json_source).expect("JSON input");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command
        .arg("--fix")
        .arg(directory.path())
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(prose).expect("fixed text"),
        "며칠 뒤에 만나요."
    );
    assert_eq!(fs::read_to_string(mdx).expect("unchanged MDX"), mdx_source);
    assert_eq!(
        fs::read_to_string(restructured_text).expect("unchanged reStructuredText"),
        rst_source
    );
    assert_eq!(
        fs::read_to_string(json).expect("unchanged JSON"),
        json_source
    );
}

#[test]
fn directory_scan_honours_geullintignore_patterns() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let checked = directory.path().join("checked.md");
    let ignored = directory.path().join("examples.md");
    let generated_directory = directory.path().join("generated");
    fs::create_dir(&generated_directory).expect("generated directory");
    fs::write(&checked, "몇일 뒤에 만나요.").expect("checked input");
    fs::write(&ignored, "몇일 뒤에 만나요.").expect("ignored input");
    fs::write(generated_directory.join("rules.md"), "몇일 뒤에 만나요.").expect("generated input");
    fs::write(
        directory.path().join(".geullintignore"),
        "examples.md\ngenerated/\n",
    )
    .expect("ignore file");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--format", "json"])
        .arg(directory.path())
        .output()
        .expect("run geullint");

    assert_eq!(output.status.code(), Some(1));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid JSON output");
    let diagnostics = document["diagnostics"]
        .as_array()
        .expect("diagnostic array");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["path"], checked.display().to_string());
}

#[test]
fn directory_scan_honours_gitignore_outside_a_git_repository() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let checked = directory.path().join("checked.md");
    let ignored = directory.path().join("ignored-by-git.md");
    fs::write(&checked, "몇일 뒤에 만나요.").expect("checked input");
    fs::write(&ignored, "몇일 뒤에 만나요.").expect("ignored input");
    fs::write(directory.path().join(".gitignore"), "ignored-by-git.md\n").expect("ignore file");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--format", "json"])
        .arg(directory.path())
        .output()
        .expect("run geullint");

    assert_eq!(output.status.code(), Some(1));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid JSON output");
    let diagnostics = document["diagnostics"]
        .as_array()
        .expect("diagnostic array");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["path"], checked.display().to_string());
}

#[test]
fn loads_disabled_rules_from_a_json_config_file() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("memo.txt");
    let config = directory.path().join("geullint.json");
    fs::write(&input, "몇일 뒤에 만나요.").expect("test input");
    fs::write(
        &config,
        r#"{"disabledRules":["spelling.lexical.myeochil"]}"#,
    )
    .expect("test config");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command.args(["--config", config.to_str().expect("UTF-8 path")]);
    command.arg(&input);
    command.assert().success().stdout("");
}

#[test]
fn loads_a_versioned_dictionary_overlay_file_without_a_network_request() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("memo.txt");
    let overlay = directory.path().join("project.overlay");
    fs::write(&input, "몇일 뒤에 만나요.").expect("test input");
    fs::write(&overlay, "geullint-overlay-v1\n몇일\tNNP\n").expect("test overlay");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command
        .args([
            "--dictionary-overlay",
            overlay.to_str().expect("UTF-8 path"),
        ])
        .arg(&input);
    command.assert().success().stdout("");
}

#[test]
fn loads_a_versioned_local_rule_pack_without_a_network_request() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("memo.txt");
    let pack = directory.path().join("team-rules.yaml");
    fs::write(&input, "글린트를 실행합니다.").expect("test input");
    fs::write(
        &pack,
        r#"
version: 1
language: ko
rules:
  - id: spelling.project.typo
    severity: warning
    message: "프로젝트 표기를 확인하세요."
    safeFix: true
    replacements:
      - from: 글린트
        to: GeulLint
"#,
    )
    .expect("test rule pack");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command
        .args(["--rule-pack", pack.to_str().expect("UTF-8 path")])
        .arg(&input);
    command
        .assert()
        .success()
        .stdout(predicates::str::contains("spelling.project.typo"))
        .stdout(predicates::str::contains("GeulLint"));
}

#[test]
fn rejects_a_malformed_local_rule_pack() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("memo.txt");
    let pack = directory.path().join("team-rules.yaml");
    fs::write(&input, "글린트를 실행합니다.").expect("test input");
    fs::write(&pack, "version: 2\nlanguage: ko\nrules: []\n").expect("test rule pack");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command
        .args(["--rule-pack", pack.to_str().expect("UTF-8 path")])
        .arg(&input)
        .assert()
        .code(2)
        .stderr(predicates::str::contains("rule pack"));
}

#[test]
fn evaluates_a_local_rule_pack_against_a_corpus() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let pack = directory.path().join("team-rules.yaml");
    let corpus = directory.path().join("team-corpus.jsonl");
    fs::write(
        &pack,
        r#"
version: 1
language: ko
rules:
  - id: spelling.project.typo
    severity: warning
    message: "프로젝트 표기를 확인하세요."
    safeFix: true
    replacements:
      - from: 글린트
        to: GeulLint
"#,
    )
    .expect("test rule pack");
    fs::write(
        &corpus,
        r#"{"id":"project-typo","text":"글린트를 실행합니다.","expectedRuleIds":["spelling.project.typo"]}"#,
    )
    .expect("test corpus");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--rule-pack", pack.to_str().expect("UTF-8 path")])
        .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
        .output()
        .expect("run geullint");

    assert!(output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid corpus JSON report");
    assert_eq!(report["truePositives"], 1);
    assert_eq!(report["falsePositives"], 0);
    assert_eq!(report["falseNegatives"], 0);
}

#[test]
fn applies_safe_fixes_from_a_local_rule_pack() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("memo.txt");
    let pack = directory.path().join("team-rules.yaml");
    fs::write(&input, "글린트를 실행합니다.").expect("test input");
    fs::write(
        &pack,
        r#"
version: 1
language: ko
rules:
  - id: spelling.project.typo
    severity: warning
    message: "프로젝트 표기를 확인하세요."
    safeFix: true
    replacements:
      - from: 글린트
        to: GeulLint
"#,
    )
    .expect("test rule pack");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command
        .arg("--fix")
        .args(["--rule-pack", pack.to_str().expect("UTF-8 path")])
        .arg(&input)
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(&input).expect("updated source"),
        "GeulLint를 실행합니다."
    );
}

#[test]
fn emits_sarif_for_code_scanning_integrations() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("memo.md");
    fs::write(&input, "몇일 뒤에 만나요.").expect("test input");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--format", "sarif"])
        .arg(&input)
        .output()
        .expect("run geullint");

    assert_eq!(output.status.code(), Some(1));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid SARIF JSON output");
    assert_eq!(document["version"], "2.1.0");
    assert_eq!(document["runs"][0]["tool"]["driver"]["name"], "GeulLint");
    assert_eq!(document["runs"][0]["columnKind"], "unicodeCodePoints");
    assert_eq!(
        document["runs"][0]["results"][0]["ruleId"],
        "spelling.lexical.myeochil"
    );
    assert_eq!(
        document["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["region"]["startLine"],
        1
    );
}

#[test]
fn sarif_keeps_repository_relative_artifact_uris_relative() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let source_directory = directory.path().join("docs");
    fs::create_dir(&source_directory).expect("source directory");
    fs::write(source_directory.join("memo.md"), "몇일 뒤에 만나자.").expect("test input");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .current_dir(directory.path())
        .args(["--format", "sarif", "docs/memo.md"])
        .output()
        .expect("run geullint");

    assert_eq!(output.status.code(), Some(1));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid SARIF JSON output");
    assert_eq!(
        document["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
        "docs/memo.md"
    );
}

#[test]
fn sarif_encodes_absolute_artifact_paths_as_file_uris() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("한글 문서 #1.md");
    fs::write(&input, "몇일 뒤에 만나자.").expect("test input");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--format", "sarif"])
        .arg(&input)
        .output()
        .expect("run geullint");

    assert_eq!(output.status.code(), Some(1));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid SARIF JSON output");
    let uri = document["runs"][0]["results"][0]["locations"][0]["physicalLocation"]
        ["artifactLocation"]["uri"]
        .as_str()
        .expect("artifact URI");

    assert!(uri.starts_with("file:///"), "unexpected URI: {uri}");
    #[cfg(windows)]
    {
        let drive = input
            .display()
            .to_string()
            .chars()
            .next()
            .expect("drive letter");
        assert!(
            uri.starts_with(&format!("file:///{drive}:/")),
            "unexpected URI: {uri}"
        );
    }
    assert!(
        uri.ends_with("/%ED%95%9C%EA%B8%80%20%EB%AC%B8%EC%84%9C%20%231.md"),
        "unexpected URI: {uri}"
    );
    assert!(!uri.contains('\\'), "unexpected URI: {uri}");
    assert!(!uri.contains(' '), "unexpected URI: {uri}");
    assert!(!uri.contains('#'), "unexpected URI: {uri}");
}

#[test]
fn fixes_only_safe_diagnostics_in_place() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("memo.txt");
    fs::write(&input, "몇일 문서를 문서를 저장합니다.").expect("test input");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command.arg("--fix").arg(&input);
    command
        .assert()
        .success()
        .stdout(predicates::str::contains("repetition.adjacent-word"));

    assert_eq!(
        fs::read_to_string(&input).expect("updated source"),
        "며칠 문서를 문서를 저장합니다."
    );
}

#[test]
fn dry_run_reports_remaining_diagnostics_without_writing_the_file() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("memo.txt");
    fs::write(&input, "몇일 문서를 문서를 저장합니다.").expect("test input");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command.arg("--fix-dry-run").arg(&input);
    command
        .assert()
        .success()
        .stdout(predicates::str::contains("repetition.adjacent-word"));

    assert_eq!(
        fs::read_to_string(&input).expect("unchanged source"),
        "몇일 문서를 문서를 저장합니다."
    );
}

#[test]
fn accepts_a_profile_from_the_command_line() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("memo.txt");
    fs::write(&input, "몇일 뒤에 만나요.").expect("test input");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command.args(["--profile", "strict"]).arg(&input);
    command
        .assert()
        .code(1)
        .stdout(predicates::str::contains("spelling.lexical.myeochil"));
}

#[test]
fn explains_a_rule_without_requiring_an_input_file() {
    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command.args(["--explain", "spelling.lexical.myeochil"]);
    command
        .assert()
        .success()
        .stdout(predicates::str::contains("spelling.lexical.myeochil"))
        .stdout(predicates::str::contains("safe"))
        .stdout(predicates::str::contains("default, strict, editorial"));
}

#[test]
fn evaluates_a_jsonl_corpus_and_emits_precision_and_recall() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let corpus = directory.path().join("corpus.jsonl");
    fs::write(
        &corpus,
        concat!(
            r#"{"id":"lexical-error","text":"몇일 뒤에 만나요.","sourceKind":"plain_text","expectedRuleIds":["spelling.lexical.myeochil"]}"#,
            "\n",
            r#"{"id":"normal-sentence","text":"며칠 뒤에 만나요.","sourceKind":"plain_text","expectedRuleIds":[]}"#,
            "\n"
        ),
    )
    .expect("test corpus");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
        .output()
        .expect("run geullint");

    assert!(output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid corpus JSON report");
    assert_eq!(report["version"], 1);
    assert_eq!(report["cases"], 2);
    assert_eq!(report["truePositives"], 1);
    assert_eq!(report["falsePositives"], 0);
    assert_eq!(report["falseNegatives"], 0);
    assert_eq!(report["precision"], 1.0);
    assert_eq!(report["recall"], 1.0);
    assert_eq!(report["normalCases"], 1);
    assert_eq!(report["falsePositiveCases"], 0);
    assert_eq!(report["specificity"], 1.0);
}

#[test]
fn reports_undefined_positive_metrics_as_null_for_normal_only_corpora() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let corpus = directory.path().join("normal-only.jsonl");
    fs::write(
        &corpus,
        r#"{"id":"normal-sentence","text":"우리 가족 관계는 오래전부터 원만했다.","sourceKind":"plain_text","expectedRuleIds":[]}"#,
    )
    .expect("normal-only corpus");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
        .output()
        .expect("run geullint");

    assert!(output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid corpus JSON report");
    assert_eq!(report["cases"], 1);
    assert_eq!(report["precision"], serde_json::Value::Null);
    assert_eq!(report["recall"], serde_json::Value::Null);
    assert_eq!(report["macroPrecision"], serde_json::Value::Null);
    assert_eq!(report["macroRecall"], serde_json::Value::Null);
    assert_eq!(report["specificity"], 1.0);
}

#[test]
fn evaluates_exact_rule_ranges_and_suggestions_when_the_corpus_supplies_them() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let corpus = directory.path().join("exact-corpus.jsonl");
    fs::write(
        &corpus,
        r#"{"id":"exact-lexical-error","text":"몇일 뒤에 만나요.","expectedDiagnostics":[{"ruleId":"spelling.lexical.myeochil","range":{"start":0,"end":6},"suggestions":["며칠"]}]}"#,
    )
    .expect("test corpus");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
        .output()
        .expect("run geullint");

    assert!(output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid corpus JSON report");
    assert_eq!(report["truePositives"], 1);
    assert_eq!(report["falsePositives"], 0);
    assert_eq!(report["falseNegatives"], 0);
}

#[test]
fn derives_an_exact_utf8_range_from_a_unique_original() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let corpus = directory.path().join("original-corpus.jsonl");
    fs::write(
        &corpus,
        r#"{"id":"original-range","text":"회의가 몇일 뒤에 열립니다.","caseType":"error","expectedDiagnostics":[{"ruleId":"spelling.lexical.myeochil","original":"몇일","suggestions":["며칠"]}],"expectedFixedText":"회의가 며칠 뒤에 열립니다."}"#,
    )
    .expect("test corpus");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
        .output()
        .expect("run geullint");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid corpus JSON report");
    assert_eq!(report["truePositives"], 1);
    assert_eq!(report["caseFailures"], serde_json::json!([]));
}

#[test]
fn rejects_an_ambiguous_original_annotation() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let corpus = directory.path().join("ambiguous-original.jsonl");
    fs::write(
        &corpus,
        r#"{"id":"ambiguous","text":"몇일 뒤 몇일 안에","caseType":"error","expectedDiagnostics":[{"ruleId":"spelling.lexical.myeochil","original":"몇일","suggestions":["며칠"]}]}"#,
    )
    .expect("test corpus");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command
        .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
        .assert()
        .code(2)
        .stderr(predicates::str::contains(
            "original must occur exactly once",
        ));
}

#[test]
fn rejects_empty_originals_and_invalid_or_inconsistent_utf8_ranges() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let fixtures = [
        (
            "empty-original.jsonl",
            r#"{"id":"empty","text":"몇일 뒤에 만나요.","expectedDiagnostics":[{"ruleId":"spelling.lexical.myeochil","original":"","suggestions":["며칠"]}]}"#,
            "empty original",
        ),
        (
            "invalid-range.jsonl",
            r#"{"id":"invalid-range","text":"몇일 뒤에 만나요.","expectedDiagnostics":[{"ruleId":"spelling.lexical.myeochil","range":{"start":1,"end":6},"suggestions":["며칠"]}]}"#,
            "invalid UTF-8 range",
        ),
        (
            "inconsistent-range.jsonl",
            r#"{"id":"inconsistent-range","text":"몇일 뒤에 만나요.","expectedDiagnostics":[{"ruleId":"spelling.lexical.myeochil","original":"뒤에","range":{"start":0,"end":6},"suggestions":["며칠"]}]}"#,
            "range does not equal original",
        ),
    ];

    for (name, contents, expected_error) in fixtures {
        let corpus = directory.path().join(name);
        fs::write(&corpus, contents).expect("test corpus");
        let mut command = Command::cargo_bin("geullint").expect("geullint binary");
        command
            .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
            .assert()
            .code(2)
            .stderr(predicates::str::contains(expected_error));
    }
}

#[test]
fn rejects_duplicate_corpus_ids_but_allows_repeated_external_texts() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let duplicate_ids = directory.path().join("duplicate-ids.jsonl");
    fs::write(
        &duplicate_ids,
        concat!(
            r#"{"id":"same","text":"첫 번째 정상 문장입니다.","expectedRuleIds":[]}"#,
            "\n",
            r#"{"id":"same","text":"두 번째 정상 문장입니다.","expectedRuleIds":[]}"#,
            "\n",
        ),
    )
    .expect("duplicate ids corpus");
    let duplicate_texts = directory.path().join("duplicate-texts.jsonl");
    fs::write(
        &duplicate_texts,
        concat!(
            r#"{"id":"one","text":"같은   문장입니다.","expectedRuleIds":[]}"#,
            "\n",
            r#"{"id":"two","text":"  같은 문장입니다.  ","expectedRuleIds":[]}"#,
            "\n",
        ),
    )
    .expect("duplicate texts corpus");

    let mut id_command = Command::cargo_bin("geullint").expect("geullint binary");
    id_command
        .args(["--corpus", duplicate_ids.to_str().expect("UTF-8 path")])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("duplicate case id"));

    let mut text_command = Command::cargo_bin("geullint").expect("geullint binary");
    text_command
        .args(["--corpus", duplicate_texts.to_str().expect("UTF-8 path")])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"cases\": 2"));
}

#[test]
fn rejects_case_type_annotation_mismatches() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let corpus = directory.path().join("case-type-mismatch.jsonl");
    fs::write(
        &corpus,
        r#"{"id":"normal-with-error","text":"몇일 뒤에 만나요.","caseType":"normal","expectedRuleIds":["spelling.lexical.myeochil"]}"#,
    )
    .expect("test corpus");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command
        .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
        .assert()
        .code(2)
        .stderr(predicates::str::contains(
            "normal caseType requires no expected diagnostics",
        ));
}

#[test]
fn reports_an_expected_fixed_text_mismatch() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let corpus = directory.path().join("wrong-fixed-text.jsonl");
    fs::write(
        &corpus,
        r#"{"id":"wrong-fix","text":"몇일 뒤에 만나요.","caseType":"error","expectedDiagnostics":[{"ruleId":"spelling.lexical.myeochil","original":"몇일","suggestions":["며칠"]}],"expectedFixedText":"몇 일 뒤에 만나요."}"#,
    )
    .expect("test corpus");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
        .output()
        .expect("run geullint");

    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid corpus JSON report");
    assert_eq!(
        report["caseFailures"][0]["fixedTextMismatch"]["expected"],
        "몇 일 뒤에 만나요."
    );
    assert_eq!(
        report["caseFailures"][0]["fixedTextMismatch"]["actual"],
        "며칠 뒤에 만나요."
    );
}

#[test]
fn accepts_unchanged_fixed_text_for_a_review_only_diagnostic() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let corpus = directory.path().join("review-only.jsonl");
    fs::write(
        &corpus,
        r#"{"id":"review-jjigae","text":"저녁에는 찌게를 먹었다.","profile":"strict","caseType":"error","expectedDiagnostics":[{"ruleId":"spelling.lexical.jjigae","original":"찌게","suggestions":["찌개"]}],"expectedFixedText":"저녁에는 찌게를 먹었다."}"#,
    )
    .expect("test corpus");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command
        .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
        .assert()
        .success();
}

#[test]
fn keeps_old_corpus_rows_backward_compatible() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let corpus = directory.path().join("legacy.jsonl");
    fs::write(
        &corpus,
        r#"{"id":"legacy","text":"몇일 뒤에 만나요.","expectedRuleIds":["spelling.lexical.myeochil"]}"#,
    )
    .expect("test corpus");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command
        .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
        .assert()
        .success();
}

#[test]
fn counts_a_wrong_exact_annotation_as_both_a_false_positive_and_false_negative() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let corpus = directory.path().join("exact-corpus.jsonl");
    fs::write(
        &corpus,
        r#"{"id":"wrong-suggestion","text":"몇일 뒤에 만나요.","expectedDiagnostics":[{"ruleId":"spelling.lexical.myeochil","range":{"start":0,"end":6},"suggestions":["몇 일"]}]}"#,
    )
    .expect("test corpus");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
        .output()
        .expect("run geullint");

    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid corpus JSON report");
    assert_eq!(report["truePositives"], 0);
    assert_eq!(report["falsePositives"], 1);
    assert_eq!(report["falseNegatives"], 1);
}

#[test]
fn reports_per_rule_precision_recall_and_wilson_lower_bound() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let corpus = directory.path().join("rule-metrics-corpus.jsonl");
    fs::write(
        &corpus,
        concat!(
            r#"{"id":"true-positive","text":"몇일 뒤에 만나요.","expectedRuleIds":["spelling.lexical.myeochil"]}"#,
            "\n",
            r#"{"id":"false-positive","text":"보고서 제출은 몇일 뒤입니다.","expectedRuleIds":[]}"#,
            "\n",
            r#"{"id":"false-negative","text":"오늘 문서를 읽는다.","expectedRuleIds":["spelling.lexical.myeochil"]}"#,
            "\n",
        ),
    )
    .expect("test corpus");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
        .output()
        .expect("run geullint");

    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid corpus JSON report");
    let metric = report["ruleMetrics"]
        .as_array()
        .expect("per-rule metrics")
        .iter()
        .find(|metric| metric["ruleId"] == "spelling.lexical.myeochil")
        .expect("myeochil metric");

    assert_eq!(metric["truePositives"], 1);
    assert_eq!(metric["falsePositives"], 1);
    assert_eq!(metric["falseNegatives"], 1);
    assert_eq!(metric["precision"], 0.5);
    assert_eq!(metric["recall"], 0.5);
    assert!(
        metric["precisionWilsonLower95"]
            .as_f64()
            .is_some_and(|value| value > 0.09 && value < 0.10)
    );
    assert_eq!(report["macroPrecision"], 0.5);
    assert_eq!(report["macroRecall"], 0.5);
}

#[test]
fn evaluates_a_local_corpus_against_declared_quality_thresholds() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let corpus = directory.path().join("quality-gate-corpus.jsonl");
    fs::write(
        &corpus,
        concat!(
            r#"{"id":"true-positive","text":"몇일 뒤에 만나요.","expectedRuleIds":["spelling.lexical.myeochil"]}"#,
            "\n",
            r#"{"id":"false-positive","text":"보고서 제출은 몇일 뒤입니다.","expectedRuleIds":[]}"#,
            "\n",
            r#"{"id":"false-negative","text":"오늘 문서를 읽는다.","expectedRuleIds":["spelling.lexical.myeochil"]}"#,
            "\n",
        ),
    )
    .expect("test corpus");
    let gate = directory.path().join("quality-gate.json");
    fs::write(
        &gate,
        r#"{"schemaVersion":1,"minMicroPrecision":0.5,"minMacroPrecision":0.5,"minRecall":0.5,"minRulePrecisionWilsonLower95":0.09,"minExpectedPerRule":2,"requiredRuleIds":["spelling.lexical.myeochil"]}"#,
    )
    .expect("quality gate");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args([
            "--corpus",
            corpus.to_str().expect("UTF-8 path"),
            "--corpus-gate",
            gate.to_str().expect("UTF-8 path"),
        ])
        .output()
        .expect("run geullint");

    assert!(output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid corpus JSON report");
    assert_eq!(report["qualityGate"]["passed"], true);
    assert_eq!(report["qualityGate"]["failures"], serde_json::json!([]));
}

#[test]
fn corpus_evaluation_fails_when_annotations_and_diagnostics_do_not_match() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let corpus = directory.path().join("corpus.jsonl");
    fs::write(
        &corpus,
        r#"{"id":"false-positive","text":"몇일 뒤에 만나요.","expectedRuleIds":[]}"#,
    )
    .expect("test corpus");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
        .output()
        .expect("run geullint");

    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid corpus JSON report");
    assert_eq!(report["falsePositives"], 1);
    assert_eq!(report["caseFailures"][0]["id"], "false-positive");
}

#[test]
fn bundled_seed_corpus_keeps_every_stable_rule_id_executable() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let corpus = workspace_root.join("corpus").join("seed-v1.jsonl");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
        .output()
        .expect("run geullint");

    assert!(output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid corpus JSON report");
    let declared_count: usize =
        fs::read_to_string(workspace_root.join("rules").join("catalog-count.txt"))
            .expect("catalog count")
            .trim()
            .parse()
            .expect("integer catalog count");
    assert_eq!(report["cases"], declared_count);
    assert!(report["truePositives"].as_u64().unwrap_or_default() >= declared_count as u64);
    assert_eq!(report["falsePositives"], 0);
    assert_eq!(report["falseNegatives"], 0);
    assert_eq!(
        report["ruleMetrics"].as_array().map(Vec::len),
        Some(declared_count)
    );
}

#[test]
fn fixes_multiple_people_text_files_without_changing_clean_context() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let first = directory.path().join("writer-a.txt");
    let second = directory.path().join("writer-b.txt");
    let clean = directory.path().join("writer-c.txt");
    fs::write(&first, "회의가 몇일 뒤에 열립니다.\n").expect("first writer");
    fs::write(&second, "저녁을 먹고 설겆이를 마쳤습니다.\n").expect("second writer");
    fs::write(&clean, "두 사람의 가족 관계가 복잡합니다.\n").expect("clean writer");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command
        .args(["--fix", directory.path().to_str().expect("UTF-8 path")])
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(first).expect("fixed first writer"),
        "회의가 며칠 뒤에 열립니다.\n"
    );
    assert_eq!(
        fs::read_to_string(second).expect("fixed second writer"),
        "저녁을 먹고 설거지를 마쳤습니다.\n"
    );
    assert_eq!(
        fs::read_to_string(clean).expect("preserved clean writer"),
        "두 사람의 가족 관계가 복잡합니다.\n"
    );
}

#[test]
fn curated_sentence_corpus_checks_real_contexts_and_normal_controls() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let corpus = workspace_root.join("corpus").join("curated-alpha-v1.jsonl");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--corpus", corpus.to_str().expect("UTF-8 path")])
        .output()
        .expect("run geullint");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid corpus JSON report");
    assert_eq!(report["cases"], 126);
    assert_eq!(report["normalCases"], 42);
    assert_eq!(report["truePositives"], 84);
    assert_eq!(report["falsePositiveCases"], 0);
    assert_eq!(report["falsePositives"], 0);
    assert_eq!(report["falseNegatives"], 0);
}

#[test]
fn evaluates_a_manifest_with_local_provenance_and_sha256() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let corpus = directory.path().join("licensed-corpus.jsonl");
    let contents = concat!(
        r#"{"id":"lexical-error","text":"몇일 뒤에 만나요.","expectedRuleIds":["spelling.lexical.myeochil"]}"#,
        "\n"
    );
    fs::write(&corpus, contents).expect("test corpus");
    let sha256 =
        Sha256::digest(contents.as_bytes())
            .iter()
            .fold(String::new(), |mut output, byte| {
                write!(output, "{byte:02x}").expect("write digest");
                output
            });
    let manifest = directory.path().join("licensed-corpus.manifest.json");
    fs::write(
        &manifest,
        format!(
            r#"{{"schemaVersion":1,"name":"fixture corpus","license":"CC-BY-4.0","sourceUrl":"https://example.invalid/corpus","corpusPath":"licensed-corpus.jsonl","sha256":"{sha256}"}}"#
        ),
    )
    .expect("test manifest");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["--corpus-manifest", manifest.to_str().expect("UTF-8 path")])
        .output()
        .expect("run geullint");

    assert!(output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid corpus JSON report");
    assert_eq!(report["provenance"]["name"], "fixture corpus");
    assert_eq!(report["provenance"]["sha256"], sha256);
}

#[test]
fn rejects_a_manifest_when_the_corpus_hash_changes() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let corpus = directory.path().join("licensed-corpus.jsonl");
    fs::write(
        &corpus,
        r#"{"id":"normal","text":"오늘 문서를 읽는다.","expectedRuleIds":[]}"#,
    )
    .expect("test corpus");
    let manifest = directory.path().join("licensed-corpus.manifest.json");
    fs::write(
        &manifest,
        r#"{"schemaVersion":1,"name":"fixture corpus","license":"CC-BY-4.0","sourceUrl":"https://example.invalid/corpus","corpusPath":"licensed-corpus.jsonl","sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#,
    )
    .expect("test manifest");

    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command
        .args(["--corpus-manifest", manifest.to_str().expect("UTF-8 path")])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("SHA-256 does not match"));
}

#[test]
fn rules_catalog_json_exposes_all_bundled_metadata_in_stable_order() {
    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["rules", "--format", "json"])
        .output()
        .expect("run geullint rules");

    assert!(output.status.success());
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid rule catalogue JSON");
    let rules = document["rules"].as_array().expect("rules array");

    assert_eq!(document["version"], 1);
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let declared_count: usize =
        fs::read_to_string(workspace_root.join("rules").join("catalog-count.txt"))
            .expect("catalog count")
            .trim()
            .parse()
            .expect("integer catalog count");
    assert_eq!(document["ruleCount"], declared_count);
    assert_eq!(rules.len(), declared_count);
    assert!(
        rules
            .windows(2)
            .all(|pair| pair[0]["id"].as_str() < pair[1]["id"].as_str())
    );
    assert!(
        rules
            .iter()
            .all(|rule| rule["documentationUrl"].as_str().is_some())
    );
}

#[test]
fn rules_catalog_markdown_contains_one_anchor_for_every_rule() {
    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    let output = command
        .args(["rules", "--format", "markdown"])
        .output()
        .expect("run geullint rules");

    assert!(output.status.success());
    let markdown = String::from_utf8(output.stdout).expect("UTF-8 Markdown catalogue");
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let declared_count: usize =
        fs::read_to_string(workspace_root.join("rules").join("catalog-count.txt"))
            .expect("catalog count")
            .trim()
            .parse()
            .expect("integer catalog count");
    assert!(markdown.starts_with(&format!("# GeulLint 규칙 {declared_count}개\n")));
    assert_eq!(markdown.matches("<a id=\"").count(), declared_count);
    assert!(markdown.contains("<a id=\"spelling.lexical.myeochil\"></a>"));
}

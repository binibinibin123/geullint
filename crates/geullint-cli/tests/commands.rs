use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use std::fs;
use std::process::Command as ProcessCommand;

#[test]
fn check_stdin_accepts_text_without_a_path() {
    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command
        .args(["check", "--stdin", "--format", "json"])
        .write_stdin("몇일 뒤에 만나요.")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("spelling.lexical.myeochil"));
}

#[test]
fn init_and_doctor_create_and_validate_local_configuration() {
    let directory = tempfile::tempdir().expect("temporary directory");
    Command::cargo_bin("geullint")
        .expect("geullint binary")
        .current_dir(directory.path())
        .args(["init"])
        .assert()
        .success();
    assert!(directory.path().join(".geullint.json").is_file());
    Command::cargo_bin("geullint")
        .expect("geullint binary")
        .current_dir(directory.path())
        .args(["doctor", "--format", "json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("configuration"));
}

#[test]
fn dictionary_validate_checks_the_local_overlay_format() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let overlay = directory.path().join("terms.overlay");
    fs::write(&overlay, "geullint-overlay-v1\nGeulLint\tNNP\n").expect("overlay");
    Command::cargo_bin("geullint")
        .expect("geullint binary")
        .args([
            "dictionary",
            "validate",
            overlay.to_str().expect("UTF-8 path"),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("1"));
}

#[test]
fn fix_alias_can_preview_a_diff_without_mutating_the_source() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("memo.txt");
    fs::write(&input, "몇일 뒤에 만나요.").expect("input");
    Command::cargo_bin("geullint")
        .expect("geullint binary")
        .current_dir(directory.path())
        .args(["fix", "--diff", "memo.txt"])
        .assert()
        .success()
        .stdout(predicates::str::contains("--- memo.txt"))
        .stdout(predicates::str::contains("-몇일 뒤에 만나요."))
        .stdout(predicates::str::contains("+며칠 뒤에 만나요."));
    assert_eq!(
        fs::read_to_string(&input).expect("source"),
        "몇일 뒤에 만나요."
    );
}

#[test]
fn cache_reuses_a_clean_content_hash_without_changing_output() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let input = directory.path().join("memo.txt");
    fs::write(&input, "며칠 뒤에 만나요.").expect("input");
    for _ in 0..2 {
        Command::cargo_bin("geullint")
            .expect("geullint binary")
            .current_dir(directory.path())
            .args(["--cache", "--format", "json", "memo.txt"])
            .assert()
            .success()
            .stdout(predicates::str::contains("\"diagnostics\""));
    }
    assert!(directory.path().join(".geullint/cache-v1.json").is_file());
}

#[test]
fn feedback_export_is_local_and_writes_jsonl_only() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let feedback_directory = directory.path().join(".geullint");
    fs::create_dir_all(&feedback_directory).expect("feedback directory");
    fs::write(
        feedback_directory.join("feedback.jsonl"),
        "{\"ruleId\":\"spelling.lexical.myeochil\",\"accepted\":true,\"text\":\"몇일\"}\nnot-json\n",
    )
    .expect("feedback");
    let output = directory.path().join("export.jsonl");
    Command::cargo_bin("geullint")
        .expect("geullint binary")
        .current_dir(directory.path())
        .args([
            "feedback",
            "export",
            "--output",
            output.to_str().expect("path"),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("1"));
    let exported = fs::read_to_string(output).expect("exported feedback");
    assert_eq!(exported.lines().count(), 1);
    assert!(!exported.contains("몇일"));
}

#[test]
fn completion_outputs_a_shell_specific_script() {
    Command::cargo_bin("geullint")
        .expect("geullint binary")
        .args(["completion", "powershell"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Register-ArgumentCompleter"));
}

#[test]
fn no_color_is_an_explicit_plain_output_contract() {
    let mut command = Command::cargo_bin("geullint").expect("geullint binary");
    command
        .args(["check", "--stdin", "--no-color", "--format", "human"])
        .write_stdin("몇일 뒤에 만나요.")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("spelling.lexical.myeochil"))
        .stdout(predicates::str::is_match("\\x1b\\[").unwrap().not());
}

#[test]
fn changed_checks_staged_worktree_and_untracked_source_files() {
    let directory = tempfile::tempdir().expect("temporary directory");
    run_git(directory.path(), ["init", "-q"]);
    run_git(
        directory.path(),
        ["config", "user.email", "tests@example.invalid"],
    );
    run_git(directory.path(), ["config", "user.name", "GeulLint Tests"]);

    let tracked = directory.path().join("tracked.txt");
    fs::write(&tracked, "며칠 뒤에 만나요.\n").expect("tracked source");
    run_git(directory.path(), ["add", "tracked.txt"]);
    run_git(directory.path(), ["commit", "-qm", "baseline"]);
    fs::write(&tracked, "몇일 뒤에 만나요.\n").expect("modified source");

    let untracked = directory.path().join("untracked.md");
    fs::write(&untracked, "몇일 뒤에 만나요.\n").expect("untracked source");

    Command::cargo_bin("geullint")
        .expect("geullint binary")
        .current_dir(directory.path())
        .args(["check", "--changed", "--format", "json"])
        .assert()
        .code(1)
        .stdout(predicates::str::contains("tracked.txt"))
        .stdout(predicates::str::contains("untracked.md"));
}

#[cfg(feature = "standard")]
#[test]
fn standard_engine_exposes_bounded_candidates_as_review_diagnostics() {
    Command::cargo_bin("geullint")
        .expect("geullint binary")
        .args([
            "check",
            "--engine",
            "standard",
            "--stdin",
            "--fail-on",
            "info",
            "--format",
            "json",
        ])
        .write_stdin("문서느 검사")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("spelling.oov.near"))
        .stdout(predicates::str::contains("\"safeFix\": false"));
}

#[cfg(feature = "standard")]
#[test]
fn context_engine_is_explicit_and_keeps_learned_candidates_review_only() {
    Command::cargo_bin("geullint")
        .expect("geullint binary")
        .args([
            "check",
            "--engine",
            "context",
            "--stdin",
            "--fail-on",
            "info",
            "--format",
            "json",
        ])
        .write_stdin("문새를 저장합니다.")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("spelling.oov.near"))
        .stdout(predicates::str::contains("\"safeFix\": false"));
}

fn run_git<const N: usize>(directory: &std::path::Path, args: [&str; N]) {
    let status = ProcessCommand::new("git")
        .args(args)
        .current_dir(directory)
        .status()
        .expect("git available");
    assert!(status.success(), "git command failed");
}

use assert_cmd::Command;
use std::fs;

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

use assert_cmd::Command;
use predicates::{prelude::PredicateBooleanExt, str::contains};

#[test]
fn test_help_output() {
    let mut cmd = Command::cargo_bin("licensa").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("Usage").and(contains("licensa")));
}

#[test]
fn test_version_output() {
    let mut cmd = Command::cargo_bin("licensa").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(contains("licensa").and(contains("1.0"))); // Adjust version as needed
}

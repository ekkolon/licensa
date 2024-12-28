use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn test_run_subcommand() {
    let mut cmd = Command::cargo_bin("licensa").unwrap();
    cmd.arg("run")
        .arg("--example")
        .assert()
        .success()
        .stdout(contains("Running with example flag"));
}

#[test]
fn test_unknown_subcommand() {
    let mut cmd = Command::cargo_bin("licensa").unwrap();
    cmd.arg("unknown")
        .assert()
        .failure()
        .stderr(contains("error: unrecognized subcommand"));
}

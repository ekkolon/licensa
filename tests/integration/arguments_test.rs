use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn test_missing_required_argument() {
    let mut cmd = Command::cargo_bin("licensa").unwrap();
    cmd.arg("run").assert().failure().stderr(contains(
        "error: The following required arguments were not provided",
    ));
}

#[test]
fn test_valid_argument() {
    let mut cmd = Command::cargo_bin("licensa").unwrap();
    cmd.arg("run")
        .arg("--config")
        .arg("config.json")
        .assert()
        .success()
        .stdout(contains("Using config: config.json"));
}

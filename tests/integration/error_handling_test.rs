use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn test_file_not_found_error() {
    let mut cmd = Command::cargo_bin("licensa").unwrap();
    cmd.arg("read")
        .arg("non_existent_file.txt")
        .assert()
        .failure()
        .stderr(contains("error: File not found"));
}

#[test]
fn test_invalid_input_error() {
    let mut cmd = Command::cargo_bin("licensa").unwrap();
    cmd.arg("run")
        .arg("--invalid")
        .assert()
        .failure()
        .stderr(contains(
            "error: Found argument '--invalid' which wasn't expected",
        ));
}

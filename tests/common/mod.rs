use assert_cmd::Command;
use predicates::str::contains;

pub fn setup_cli() -> Command {
    Command::cargo_bin("licensa").expect("Binary not found")
}

pub fn assert_error_output(command: &mut Command, expected_msg: &str) {
    command.assert().failure().stderr(contains(expected_msg));
}

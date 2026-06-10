use assert_cmd::Command;
use predicates::str::contains;

fn cmd() -> Command {
    Command::cargo_bin("paddle").unwrap()
}

#[test]
fn runs_a_file() {
    cmd()
        .args(["examples/fact.pd"])
        .current_dir("..")
        .assert()
        .success()
        .stdout(contains("3628800"));
}

#[test]
fn no_std_still_evaluates_builtins() {
    cmd()
        .args(["--no-std", "examples/fact.pd"])
        .current_dir("..")
        .assert()
        .success()
        .stdout(contains("3628800"));
}

#[test]
fn stdin_evaluates_expression() {
    cmd()
        .args(["--no-std"])
        .write_stdin("(+ 1 2)\n")
        .assert()
        .success()
        .stdout(contains("3"));
}

#[test]
fn missing_file_exits_nonzero() {
    cmd().args(["nonexistent.pd"]).assert().failure();
}

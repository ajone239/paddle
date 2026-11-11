use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let status = Command::new("hyperfine")
        .arg("--export-json")
        .arg("blah.json")
        .arg("--warmup")
        .arg("2")
        .arg("--runs")
        .arg("3")
        .arg("--prepare")
        .arg("cargo build --release")
        .arg("paddle ./examples/nqueens.pd")
        .arg("./target/release/paddle ./examples/nqueens.pd")
        .arg("paddle ./examples/sudoku.pd")
        .arg("./target/release/paddle ./examples/sudoku.pd")
        .status()
        .expect("it fugged up");

    if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

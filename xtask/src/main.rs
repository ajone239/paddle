use clap::Parser;
use std::process::{Command, ExitCode};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Specify if you want to export
    #[arg(short = 'x', long)]
    export: bool,

    #[arg(short, long)]
    export_file: Option<String>,

    /// Specify the example file you want to benchmark
    benches: Vec<String>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let benches = if cli.benches.is_empty() {
        vec!["examples/nqueens.pd".to_owned()]
    } else {
        cli.benches
    };

    let mut status = true;

    for bench in benches {
        let export_file = format!(
            "{}_{}",
            bench.replace('/', "-"),
            cli.export_file.clone().unwrap_or("bench.json".to_string())
        );
        status &= run_profiler(&bench, &export_file, cli.export);
    }

    if status {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run_profiler(filename: &str, export_file: &str, export: bool) -> bool {
    let mut args = vec![];

    if export {
        args.push("--export-json");
        args.push(export_file);
    }

    args.extend(vec![
        "--warmup",
        "2",
        "--runs",
        "3",
        "--prepare",
        "cargo build --release",
    ]);

    let old = format!("paddle {}", filename);
    let new = format!("./target/release/paddle {}", filename);

    args.push(&old);
    args.push(&new);

    Command::new("hyperfine")
        .args(args)
        .status()
        .expect("Running hyperfine failed.")
        .success()
}

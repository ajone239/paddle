use std::{cell::RefCell, path::PathBuf, rc::Rc};

use anyhow::{Context, Result};
use clap::Parser;

use paddle::repl::run_repl;
use paddle_core::{
    cursor::{display_results, process, process_file},
    eval::Env,
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Specify the file to run
    file: Option<PathBuf>,

    /// runs the repl
    #[arg(short, long)]
    repl: bool,

    /// Skips std-lib
    #[arg(short, long)]
    no_std: bool,
}

static STD_LIB: &str = include_str!("../../examples/base.pd");
static STD_MAC: &str = include_str!("../../examples/macros.pd");

fn main() -> Result<()> {
    let cli = Cli::parse();
    let env = Rc::new(RefCell::new(Env::default()));

    if !cli.no_std {
        process(STD_LIB, env.clone()).context("failed to parse the std lib")?;
        process(STD_MAC, env.clone()).context("failed to parse the std lib macros")?;
    }

    if let Some(file_path) = cli.file.clone() {
        let res = process_file(file_path, env.clone());
        display_results(res);
    }

    if cli.repl || cli.file.is_none() {
        run_repl(env)?
    }

    Ok(())
}

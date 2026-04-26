use std::{cell::RefCell, path::PathBuf, rc::Rc};

use anyhow::{Context, Result};
use clap::{Parser, command};

use paddle::{
    cursor::{display_results, process, process_file},
    eval::Env,
    repl::run_repl,
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Specify the file to run
    file: Option<PathBuf>,
}

static STD_LIB: &'static str = include_str!("../examples/base.pd");

fn main() -> Result<()> {
    let cli = Cli::parse();
    let env = Rc::new(RefCell::new(Env::default()));

    process(STD_LIB, env.clone()).context("failed to parse the std lib")?;

    match cli.file {
        Some(file_path) => {
            let res = process_file(file_path, env);
            display_results(res);
        }
        None => run_repl(env)?,
    }

    Ok(())
}

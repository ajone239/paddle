use std::{cell::RefCell, fs::read_to_string, path::PathBuf, rc::Rc};

use anyhow::Result;
use clap::Parser;

use paddle::repl::run_repl;
use paddle_core::{
    cursor::{Cursor, display_result},
    eval::Env,
    lexer,
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let env = if !cli.no_std {
        Env::with_stdlib()?
    } else {
        Rc::new(RefCell::new(Env::default()))
    };

    if let Some(file_path) = cli.file.clone() {
        let contents = read_to_string(file_path)?;
        let lexed = lexer::lex(&contents);

        let cursor = Cursor::new(&lexed, env.clone());

        for r in cursor {
            display_result(r);
        }
    }

    if cli.repl || cli.file.is_none() {
        run_repl(env)?
    }

    Ok(())
}

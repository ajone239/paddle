use std::{cell::RefCell, fs::read_to_string, path::PathBuf, rc::Rc};

use anyhow::Result;
use clap::{Parser, command};
use rustyline::{DefaultEditor, error::ReadlineError};

use paddle::{
    eval::{Env, eval, lower, value::Value},
    lexer, parser,
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Specify the file to run
    file: Option<PathBuf>,
}

static STD_LIB: &'static str = include_str!("../examples/base.jt");

fn main() -> Result<()> {
    let cli = Cli::parse();
    let env = Rc::new(RefCell::new(Env::default()));

    process_file(STD_LIB, env.clone(), false);

    if let Some(file_path) = cli.file {
        let contents = read_to_string(file_path)?;
        let env = Rc::new(RefCell::new(Env::default()));

        process_file(&contents, env, true);
        return Ok(());
    }

    run_repl(env)?;

    Ok(())
}

fn process_file(contents: &str, env: Rc<RefCell<Env>>, print: bool) {
    let mut from = 0;
    for i in 0..contents.len() {
        let chunk = &contents[from..i + 1];

        let p = count_paren(&chunk);

        if p != 0 || i == from {
            continue;
        }

        from = i + 1;

        let value = lpe(chunk, env.clone());
        if print {
            display_value(value);
        }
    }
}

fn run_repl(env: Rc<RefCell<Env>>) -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    let mut input = String::new();

    loop {
        let pcount = count_paren(&input);
        let prompt = make_prompt(pcount as usize);

        let line = match rl.readline(&prompt) {
            Ok(line) => line,
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => break,
            Err(e) => return Err(e.into()),
        };

        let line = line.trim();

        if input.is_empty() {
            let repl_cmd = match line {
                ":env" => {
                    env.borrow().dump();
                    true
                }
                ":env_debug" => {
                    println!("{:#?}", env.borrow());
                    true
                }
                s if s.starts_with(":require ") => {
                    s.split_whitespace()
                        .skip(1)
                        .for_each(|f| match read_to_string(f) {
                            Ok(contents) => process_file(&contents, env.clone(), false),
                            Err(err) => println!("Problem reading {}: {:?}", f, err),
                        });
                    true
                }
                _ => false,
            };

            if repl_cmd {
                rl.add_history_entry(line)?;
                continue;
            }
        }

        input += line;

        let pcount = count_paren(&input);

        match pcount {
            c if c < 0 => println!("Err: bad paren structure!"),
            c if c > 0 => continue,
            _ => {
                rl.add_history_entry(&input)?;
                let val = lpe(&input, env.clone());
                display_value(val);
            }
        }

        input.clear();
    }

    Ok(())
}

fn lpe(input: &str, env: Rc<RefCell<Env>>) -> Result<Value> {
    let tokens = lexer::lex(input);
    let (ast, _) = parser::parse_expr(&tokens)?;
    let expr = lower(&ast);
    eval(&expr, env)
}

fn count_paren(line: &str) -> i32 {
    line.chars()
        .map(|c| match c {
            '(' => 1,
            ')' => -1,
            _ => 0,
        })
        .sum()
}

fn display_value(val: Result<Value>) {
    match val {
        Ok(Value::Nil) => {}
        Ok(val) => println!("{}", val),
        Err(err) => println!("ERROR: {:?}", err),
    }
}

fn make_prompt(indent: usize) -> String {
    if indent == 0 {
        "> ".to_string()
    } else {
        let p = "  ".repeat(indent);
        format!("* {}", p)
    }
}

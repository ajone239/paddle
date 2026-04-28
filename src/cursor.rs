use anyhow::{Result, bail};

use std::{cell::RefCell, fs::read_to_string, path::PathBuf, rc::Rc};

use crate::{
    eval::{Env, eval, lower, value::Value},
    lexer, parser,
};

pub fn process_file(file_path: PathBuf, env: Rc<RefCell<Env>>) -> Result<Vec<Value>> {
    let contents = read_to_string(file_path)?;
    process(&contents, env)
}

pub fn process(contents: &str, env: Rc<RefCell<Env>>) -> Result<Vec<Value>> {
    if contents.trim().is_empty() {
        return Ok(vec![]);
    }

    let tokens = lexer::lex(contents);

    let mut working = &tokens[..];

    let mut rv = vec![];

    loop {
        let (ast, rest) = parser::parse_expr(&working)?;
        let expr = lower(&ast);
        let val = eval(&expr, env.clone())?;

        rv.push(val);

        if rest.is_empty() {
            break;
        }

        working = rest;
    }

    Ok(rv)
}

pub fn display_results(res: Result<Vec<Value>>) {
    match res {
        Err(err) => println!("ERROR: {:?}", err),
        Ok(vals) => {
            for val in vals {
                println!("{}", val);
            }
        }
    }
}

pub fn is_ready_to_process(contents: &str) -> Result<bool> {
    let p = count_paren(contents);

    match p {
        c if c < 0 => bail!("More closing than opening parens."),
        c if c > 0 => Ok(false),
        _ => Ok(true),
    }
}

pub fn count_paren(line: &str) -> i32 {
    line.chars()
        .map(|c| match c {
            '(' => 1,
            ')' => -1,
            _ => 0,
        })
        .sum()
}

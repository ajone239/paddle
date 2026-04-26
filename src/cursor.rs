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
    let mut from = 0;
    let mut rv = vec![];

    for i in 0..contents.len() {
        let chunk = &contents[from..i + 1];

        let p = count_paren(&chunk);

        if p != 0 || i == from || chunk.trim().len() == 0 {
            continue;
        }

        from = i + 1;

        let value = lpe(chunk, env.clone())?;

        rv.push(value);
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
        c if c < 0 => bail!("frick"),
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

fn lpe(input: &str, env: Rc<RefCell<Env>>) -> Result<Value> {
    let tokens = lexer::lex(input);
    let (ast, _) = parser::parse_expr(&tokens)?;
    let expr = lower(&ast);
    eval(&expr, env)
}

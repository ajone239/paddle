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

fn lpe(input: &str, env: Rc<RefCell<Env>>) -> Result<Value> {
    let tokens = lexer::lex(input);
    // TODO(ajone239): use the rest from the parser to do the processing
    let (ast, _) = parser::parse_expr(&tokens)?;
    let expr = lower(&ast);
    eval(&expr, env)
}

pub fn process(contents: &str, env: Rc<RefCell<Env>>) -> Result<Vec<Value>> {
    let mut from = 0;
    let mut rv = vec![];

    let mut p = 0;

    for (i, c) in contents.char_indices() {
        let has_no_p = p == 0;
        let is_ws = c.is_whitespace();
        let eof = i == contents.len() - 1;

        p += match c {
            '(' => 1,
            ')' => -1,
            _ => 0,
        };

        let naked_atom = has_no_p && (is_ws || c == '(' || eof);
        let good_expr = p == 0 && !has_no_p;

        let (chunk, new_from) = if naked_atom && c == '(' {
            (&contents[from..i], i)
        } else {
            (&contents[from..i + 1], i + 1)
        };

        if chunk.trim().is_empty() || (!naked_atom && !good_expr) {
            continue;
        }

        from = new_from;

        let value = lpe(chunk, env.clone())?;

        rv.push(value);
    }

    if from < contents.len() - 1 {
        bail!(
            "Full content was not parsed ({}/{}): got {} expressions",
            from,
            contents.len(),
            rv.len()
        );
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

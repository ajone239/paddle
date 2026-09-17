use anyhow::{Result, anyhow, bail};

use std::{cell::RefCell, fs::read_to_string, path::PathBuf, rc::Rc};

use crate::{
    eval::{Env, eval, lower, value::Value},
    lexer::{self, Token},
    parser,
    span::{FileId, intern},
};

pub struct Cursor<'a> {
    working: &'a [Token<'a>],
    env: Rc<RefCell<Env>>,
}

impl<'a> Cursor<'a> {
    pub fn new(working: &'a [Token<'a>], env: Rc<RefCell<Env>>) -> Self {
        Self { working, env }
    }
}

impl<'a> Iterator for Cursor<'a> {
    type Item = Result<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.working.is_empty() {
            return None;
        }

        let parse_res = parser::parse_expr(self.working);

        let Ok((ast, rest)) = parse_res else {
            self.working = &[];
            return Some(Err(parse_res.err().unwrap().into()));
        };

        let start_span = self.working[0].span;
        self.working = rest;

        let expr = lower(&ast);

        let val = eval(expr, self.env.clone()).map_err(|e| e.context(format!("at {}", start_span)));

        Some(val)
    }
}

pub fn process_file(file_path: PathBuf, env: Rc<RefCell<Env>>) -> Result<Vec<Value>> {
    let contents = read_to_string(&file_path)?;
    let file_id = intern(
        file_path
            .to_str()
            .ok_or(anyhow!(format!("Bad file {:?}", file_path)))?
            .to_owned(),
    );
    let lexed = lexer::lex(&contents, file_id);

    let cursor = Cursor::new(&lexed, env);
    cursor.collect()
}

pub fn process_named_bytes(
    file_path: PathBuf,
    contents: &str,
    env: Rc<RefCell<Env>>,
) -> Result<Vec<Value>> {
    let file_id = intern(
        file_path
            .to_str()
            .ok_or(anyhow!(format!("Bad file {:?}", file_path)))?
            .to_owned(),
    );
    let lexed = lexer::lex(&contents, file_id);

    let cursor = Cursor::new(&lexed, env);
    cursor.collect()
}

pub fn process(contents: &str, env: Rc<RefCell<Env>>) -> Result<Vec<Value>> {
    let file_id = FileId::default();
    let lexed = lexer::lex(contents, file_id);

    let cursor = Cursor::new(&lexed, env);
    cursor.collect()
}

pub fn display_result(res: Result<Value>) {
    match res {
        Err(err) => println!("ERROR: {:?}", err),
        Ok(Value::NoPrint) => {}
        Ok(val) => println!("{}", val),
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

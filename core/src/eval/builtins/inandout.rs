use std::{
    io::{self, ErrorKind, Read},
    rc::Rc,
};

use anyhow::Result;

use crate::{eval::value::Value, lexer::Span};

pub fn print(args: &Value) -> Result<Value> {
    let out = args
        .to_cons_iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    println!("{}", out);

    Ok(Value::NoPrint)
}

pub fn getchar(_args: &Value) -> Result<Value> {
    let mut buf = [0u8; 1];

    let res = io::stdin().read_exact(&mut buf);

    match res {
        Ok(_) => Ok(Value::Char(buf[0], Span::default())),
        Err(err) if err.kind() == ErrorKind::UnexpectedEof => Ok(Value::Cons(
            Rc::new((
                Value::Symbol("err".into(), Span::default()),
                Value::Str("EOF".into(), Span::default()),
            )),
            Span::default(),
        )),
        Err(err) => Err(err.into()),
    }
}

pub fn getline(_args: &Value) -> Result<Value> {
    let mut buf = String::new();
    let res = io::stdin().read_line(&mut buf);

    match res {
        Ok(_) => Ok(Value::Str(buf.trim().into(), Span::default())),
        Err(err) => Err(err.into()),
    }
}

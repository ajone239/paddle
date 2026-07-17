use crate::{eval::value::Value, lexer::Span};
use anyhow::{Result, bail};

pub fn is_number(args: &Value) -> Result<Value> {
    let Value::Cons(args, _) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_, _) = &args.1 {
        bail!("only one arg");
    };

    match args.0 {
        Value::Num(_, _) => Ok(Value::Bool(true, Span::default())),
        _ => Ok(Value::Bool(false, Span::default())),
    }
}

pub fn is_bool(args: &Value) -> Result<Value> {
    let Value::Cons(args, _) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_, _) = &args.1 {
        bail!("only one arg");
    };

    match args.0 {
        Value::Bool(_, _) => Ok(Value::Bool(true, Span::default())),
        _ => Ok(Value::Bool(false, Span::default())),
    }
}

pub fn is_symbol(args: &Value) -> Result<Value> {
    let Value::Cons(args, _) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_, _) = &args.1 {
        bail!("only one arg");
    };

    match args.0 {
        Value::Symbol(_, _) => Ok(Value::Bool(true, Span::default())),
        _ => Ok(Value::Bool(false, Span::default())),
    }
}

pub fn is_char(args: &Value) -> Result<Value> {
    let Value::Cons(args, _) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_, _) = &args.1 {
        bail!("only one arg");
    };

    match args.0 {
        Value::Char(_, _) => Ok(Value::Bool(true, Span::default())),
        _ => Ok(Value::Bool(false, Span::default())),
    }
}

pub fn is_string(args: &Value) -> Result<Value> {
    let Value::Cons(args, _) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_, _) = &args.1 {
        bail!("only one arg");
    };

    match args.0 {
        Value::Str(_, _) => Ok(Value::Bool(true, Span::default())),
        _ => Ok(Value::Bool(false, Span::default())),
    }
}

pub fn is_atom(args: &Value) -> Result<Value> {
    let Value::Cons(args, _) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_, _) = &args.1 {
        bail!("only one arg");
    };

    match args.0 {
        Value::Cons(_, _) => Ok(Value::Bool(false, Span::default())),
        _ => Ok(Value::Bool(true, Span::default())),
    }
}

pub fn is_null(args: &Value) -> Result<Value> {
    let Value::Cons(args, _) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_, _) = &args.1 {
        bail!("only one arg");
    };

    match &args.0 {
        Value::Nil(_) => Ok(Value::Bool(true, Span::default())),
        _ => Ok(Value::Bool(false, Span::default())),
    }
}

pub fn is_pair(args: &Value) -> Result<Value> {
    let Value::Cons(args, _) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_, _) = &args.1 {
        bail!("only one arg");
    };

    match args.0 {
        Value::Cons(_, _) => Ok(Value::Bool(true, Span::default())),
        _ => Ok(Value::Bool(false, Span::default())),
    }
}

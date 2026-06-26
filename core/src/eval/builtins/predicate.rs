use crate::eval::value::Value;
use anyhow::{Result, bail};

pub fn is_number(args: &Value) -> Result<Value> {
    let Value::Cons(args) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_) = &args.1 {
        bail!("only one arg");
    };

    match args.0 {
        Value::Num(_) => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

pub fn is_bool(args: &Value) -> Result<Value> {
    let Value::Cons(args) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_) = &args.1 {
        bail!("only one arg");
    };

    match args.0 {
        Value::Bool(_) => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

pub fn is_symbol(args: &Value) -> Result<Value> {
    let Value::Cons(args) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_) = &args.1 {
        bail!("only one arg");
    };

    match args.0 {
        Value::Symbol(_) => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

pub fn is_char(args: &Value) -> Result<Value> {
    let Value::Cons(args) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_) = &args.1 {
        bail!("only one arg");
    };

    match args.0 {
        Value::Char(_) => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

pub fn is_string(args: &Value) -> Result<Value> {
    let Value::Cons(args) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_) = &args.1 {
        bail!("only one arg");
    };

    match args.0 {
        Value::Str(_) => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

pub fn is_atom(args: &Value) -> Result<Value> {
    let Value::Cons(args) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_) = &args.1 {
        bail!("only one arg");
    };

    match args.0 {
        Value::Cons(_) => Ok(Value::Bool(false)),
        _ => Ok(Value::Bool(true)),
    }
}

pub fn is_null(args: &Value) -> Result<Value> {
    let Value::Cons(args) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_) = &args.1 {
        bail!("only one arg");
    };

    match &args.0 {
        Value::Nil => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

pub fn is_pair(args: &Value) -> Result<Value> {
    let Value::Cons(args) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_) = &args.1 {
        bail!("only one arg");
    };

    match args.0 {
        Value::Cons(_) => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

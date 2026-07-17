use crate::{
    eval::{env::BuiltinError, value::Value},
    lexer::Span,
};
use anyhow::{Result, bail};

pub fn and(args: &Value) -> Result<Value> {
    let mut hold = args;
    let mut pass = true;

    while let Value::Cons(pair, _) = hold {
        pass &= pair.0.truthy();
        hold = &pair.1;
    }

    Ok(Value::Bool(pass, Span::default()))
}

pub fn val_or(args: &Value) -> Result<Value> {
    let mut hold = args;
    let mut pass = false;

    while let Value::Cons(pair, _) = hold {
        pass |= pair.0.truthy();
        hold = &pair.1;
    }

    Ok(Value::Bool(pass, Span::default()))
}

pub fn eq(args: &Value) -> Result<Value> {
    let Value::Cons(pair, _) = args else {
        bail!("should give me a list");
    };

    let Value::Cons(pair2, _) = &pair.1 else {
        bail!("should give me a list");
    };

    if let Value::Cons(_, _) = pair2.1 {
        return Err(BuiltinError::BadEqArgCount.into());
    }

    match (&pair.0, &pair2.0) {
        (Value::Num(last, _), Value::Num(penu, _)) => {
            Ok(Value::Bool(penu == last, Span::default()))
        }
        (Value::Nil(_), Value::Nil(_)) => Ok(Value::Bool(true, Span::default())),
        (_, Value::Nil(_)) | (Value::Nil(_), _) => Ok(Value::Bool(false, Span::default())),
        (Value::Char(last, _), Value::Char(penu, _)) => {
            Ok(Value::Bool(penu == last, Span::default()))
        }
        (Value::Char(byte, _), Value::Str(s, _))
        | (Value::Char(byte, _), Value::Symbol(s, _))
        | (Value::Str(s, _), Value::Char(byte, _))
        | (Value::Symbol(s, _), Value::Char(byte, _)) => {
            if s.len() != 1 {
                return Ok(Value::Bool(false, Span::default()));
            }

            let s = s.bytes().next().unwrap();
            Ok(Value::Bool(s == *byte, Span::default()))
        }
        (Value::Str(last, _), Value::Str(penu, _))
        | (Value::Symbol(last, _), Value::Str(penu, _))
        | (Value::Str(last, _), Value::Symbol(penu, _))
        | (Value::Symbol(last, _), Value::Symbol(penu, _)) => {
            Ok(Value::Bool(penu == last, Span::default()))
        }
        _ => Ok(Value::Bool(false, Span::default())),
    }
}

pub fn not(args: &Value) -> Result<Value> {
    let Value::Cons(pair, _) = args else {
        bail!("should give me a list");
    };

    if let Value::Cons(_, _) = pair.1 {
        return Err(BuiltinError::WrongNotArgCount.into());
    }

    Ok(Value::Bool(!pair.0.truthy(), Span::default()))
}

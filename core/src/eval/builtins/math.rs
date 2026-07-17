use anyhow::{Result, bail};

use crate::{
    eval::{env::BuiltinError, value::Value},
    lexer::Span,
};

pub fn add(args: &Value) -> Result<Value> {
    if !matches!(args, Value::Cons(_, _) | Value::Nil(_)) {
        bail!("should give me a list");
    }

    let mut num = 0.0;

    let mut hold = args;

    while let Value::Cons(pair, _) = hold {
        match pair.0 {
            Value::Num(val, _) => {
                num += val;
            }
            Value::Nil(_) => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Num(num, Span::default()))
}

pub fn min(args: &Value) -> Result<Value> {
    let Value::Cons(pair, _) = args else {
        return Err(BuiltinError::NoInitforMinus.into());
    };

    let mut num = match pair.0 {
        Value::Num(num, _) => num,
        Value::Nil(_) => {
            return Err(BuiltinError::NoInitforMinus.into());
        }
        _ => {
            return Err(BuiltinError::ExpectedNumArg.into());
        }
    };

    let mut hold = &pair.1;

    while let Value::Cons(pair, _) = hold {
        match pair.0 {
            Value::Num(val, _) => {
                num -= val;
            }
            Value::Nil(_) => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Num(num, Span::default()))
}

pub fn mul(args: &Value) -> Result<Value> {
    if !matches!(args, Value::Cons(_, _) | Value::Nil(_)) {
        return Err(BuiltinError::NoInitforDiv.into());
    }

    let mut num = 1.0;

    let mut hold = args;

    while let Value::Cons(pair, _) = hold {
        match pair.0 {
            Value::Num(val, _) => {
                num *= val;
            }
            Value::Nil(_) => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Num(num, Span::default()))
}

pub fn div(args: &Value) -> Result<Value> {
    let Value::Cons(pair, _) = args else {
        return Err(BuiltinError::NoInitforDiv.into());
    };

    let mut num = match pair.0 {
        Value::Num(num, _) => num,
        Value::Nil(_) => {
            return Err(BuiltinError::NoInitforDiv.into());
        }
        _ => {
            return Err(BuiltinError::ExpectedNumArg.into());
        }
    };
    let mut hold = &pair.1;

    while let Value::Cons(pair, _) = hold {
        match pair.0 {
            Value::Num(val, _) => {
                num /= val;
            }
            Value::Nil(_) => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Num(num, Span::default()))
}

pub fn intdiv(args: &Value) -> Result<Value> {
    let Value::Cons(pair, _) = args else {
        return Err(BuiltinError::NoInitforDiv.into());
    };

    let mut num = match pair.0 {
        Value::Num(num, _) => num,
        Value::Nil(_) => {
            return Err(BuiltinError::NoInitforDiv.into());
        }
        _ => {
            return Err(BuiltinError::ExpectedNumArg.into());
        }
    };
    let mut hold = &pair.1;

    while let Value::Cons(pair, _) = hold {
        match pair.0 {
            Value::Num(val, _) => {
                let new_num = num.div_euclid(val);
                num = new_num;
            }
            Value::Nil(_) => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Num(num, Span::default()))
}

pub fn lt(args: &Value) -> Result<Value> {
    let Value::Cons(pair, _) = args else {
        return Err(BuiltinError::BadLtArgTypes.into());
    };

    let Value::Num(mut num, _) = pair.0 else {
        return Err(BuiltinError::ExpectedNumArg.into());
    };
    let mut hold = &pair.1;
    let mut pass = true;

    while let Value::Cons(pair, _) = hold {
        match pair.0 {
            Value::Num(val, _) => {
                pass &= num < val;
                num = val
            }
            Value::Nil(_) => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Bool(pass, Span::default()))
}

pub fn modulo(args: &Value) -> Result<Value> {
    let Value::Cons(pair, _) = args else {
        bail!("should give me a list");
    };

    let Value::Cons(pair2, _) = &pair.1 else {
        bail!("should give me a list");
    };

    if let Value::Cons(_, _) = pair2.1 {
        return Err(BuiltinError::BadModArgCount.into());
    }

    match (&pair2.0, &pair.0) {
        (Value::Num(last, _), Value::Num(penu, _)) => Ok(Value::Num(penu % last, Span::default())),
        _ => Err(BuiltinError::BadModArgTypes.into()),
    }
}

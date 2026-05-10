use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::{Ok, Result, bail};
use thiserror::Error;

use crate::eval::value::{Builtin, BuiltinFn, Value};

#[derive(Debug, PartialEq)]
pub struct Env {
    env: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Self>>>,
}

impl Env {
    pub fn new_child(parent: Rc<RefCell<Self>>) -> Self {
        Self {
            env: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.env.insert(name.to_owned(), value);
    }

    pub fn resolve(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.env.get(name) {
            return Some(val.clone());
        }

        match &self.parent {
            None => None,
            Some(penv) => penv.borrow().resolve(name),
        }
    }

    pub fn dump(&self) {
        let venv: Vec<_> = self.env.iter().collect();

        let b = venv.iter().filter(|(_, v)| matches!(v, Value::Builtin(..)));

        println!("Built-Ins:");
        for (k, v) in b {
            println!("{}: {}", k, v)
        }

        let l = venv
            .iter()
            .filter(|(_, v)| matches!(v, Value::Lambda { .. }));

        println!();
        println!("Lambdas:");
        for (k, v) in l {
            println!("{}: {}", k, v)
        }

        let f = venv.iter().filter(|(_, v)| matches!(v, Value::Func { .. }));

        println!();
        println!("Functions:");
        for (k, v) in f {
            println!("{}: {}", k, v)
        }

        let m = venv
            .iter()
            .filter(|(_, v)| matches!(v, Value::Macro { .. }));

        println!();
        println!("Macros:");
        for (k, v) in m {
            println!("{}: {}", k, v)
        }

        let r = venv.iter().filter(|(_, v)| {
            !(matches!(v, Value::Builtin(..))
                || matches!(v, Value::Lambda { .. })
                || matches!(v, Value::Func { .. })
                || matches!(v, Value::Macro { .. }))
        });

        println!();
        println!("Values:");
        for (k, v) in r {
            println!("{}: {}", k, v)
        }
        println!();
    }
}

impl Default for Env {
    fn default() -> Self {
        let mut env = HashMap::new();

        let bins: &[(&str, Builtin)] = &[
            ("+", add),
            ("*", mul),
            ("-", min),
            ("/", div),
            ("=", eq),
            ("<", lt),
            ("%", modulo),
            ("&&", and),
            ("||", val_or),
            ("not", not),
            ("cons", cons),
            ("car", car),
            ("cdr", cdr),
            ("list", list),
            ("print", print),
        ];

        for (name, f) in bins {
            env.insert(name.to_string(), tobi(*f, name));
        }

        Self { env, parent: None }
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum BuiltinError {
    #[error("Not: Expected 1 argument.")]
    WrongNotArgCount,
    #[error("Cons: Expected 1 argument.")]
    WrongConsArgCount,
    #[error("Car: Expected 1 argument.")]
    WrongCarArgCount,
    #[error("Car: Must be applied to a list.")]
    WrongCarArgType,
    #[error("Cdr: Expected 1 argument.")]
    WrongCdrArgCount,
    #[error("Cdr: Must be applied to a list.")]
    WrongCdrArgType,
    #[error("Cdr: Cannot be applied to an empty list.")]
    CdrOnEmptyList,
    #[error("Expected Number for arithmetic builtin.")]
    ExpectedNumArg,
    #[error("Minus: initial argument required.")]
    NoInitforMinus,
    #[error("LessThan: Expected Numbers")]
    BadLtArgTypes,
    #[error("Div: initial argument required.")]
    NoInitforDiv,
    #[error("Car: Cannot be applied to an empty list.")]
    CarOnEmptyList,
    #[error("Modulo: Expected numbers")]
    BadModArgTypes,
    #[error("Modulo: Expected 2 arguments")]
    BadModArgCount,
    #[error("Eq: Expected comparable types")]
    BadEqArgTypes,
    #[error("Eq: Expected 2 arguments")]
    BadEqArgCount,
}

fn tobi(f: Builtin, name: &str) -> Value {
    Value::Builtin(BuiltinFn(f), name.to_owned())
}

fn add(args: &Value) -> Result<Value> {
    if !matches!(args, Value::Cons(_)) {
        bail!("should give me a list");
    }

    let mut num = 0.0;

    let mut hold = args;

    while let Value::Cons(pair) = hold {
        match pair.0 {
            Value::Num(val) => {
                num += val;
            }
            Value::Nil => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Num(num))
}

fn min(args: &Value) -> Result<Value> {
    let Value::Cons(pair) = args else {
        return Err(BuiltinError::NoInitforMinus.into());
    };

    let mut num = match pair.0 {
        Value::Num(num) => num,
        Value::Nil => {
            return Err(BuiltinError::NoInitforMinus.into());
        }
        _ => {
            return Err(BuiltinError::ExpectedNumArg.into());
        }
    };

    let mut hold = &pair.1;

    while let Value::Cons(pair) = hold {
        match pair.0 {
            Value::Num(val) => {
                num -= val;
            }
            Value::Nil => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Num(num))
}

fn mul(args: &Value) -> Result<Value> {
    if !matches!(args, Value::Cons(_)) {
        return Err(BuiltinError::NoInitforDiv.into());
    }

    let mut num = 1.0;

    let mut hold = args;

    while let Value::Cons(pair) = hold {
        match pair.0 {
            Value::Num(val) => {
                num *= val;
            }
            Value::Nil => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Num(num))
}

fn div(args: &Value) -> Result<Value> {
    let Value::Cons(pair) = args else {
        bail!("should give me a list");
    };

    let mut num = match pair.0 {
        Value::Num(num) => num,
        Value::Nil => {
            return Err(BuiltinError::NoInitforDiv.into());
        }
        _ => {
            return Err(BuiltinError::ExpectedNumArg.into());
        }
    };
    let mut hold = &pair.1;

    while let Value::Cons(pair) = hold {
        match pair.0 {
            Value::Num(val) => {
                num /= val;
            }
            Value::Nil => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Num(num))
}

fn lt(args: &Value) -> Result<Value> {
    let Value::Cons(pair) = args else {
        return Err(BuiltinError::BadLtArgTypes.into());
    };

    let Value::Num(mut num) = pair.0 else {
        return Err(BuiltinError::ExpectedNumArg.into());
    };
    let mut hold = &pair.1;
    let mut pass = true;

    while let Value::Cons(pair) = hold {
        match pair.0 {
            Value::Num(val) => {
                pass &= num < val;
                num = val
            }
            Value::Nil => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Bool(pass))
}

fn and(args: &Value) -> Result<Value> {
    let mut hold = args;
    let mut pass = true;

    while let Value::Cons(pair) = hold {
        pass &= pair.0.truthy();
        hold = &pair.1;
    }

    Ok(Value::Bool(pass))
}

fn val_or(args: &Value) -> Result<Value> {
    let mut hold = args;
    let mut pass = true;

    while let Value::Cons(pair) = hold {
        pass |= pair.0.truthy();
        hold = &pair.1;
    }

    Ok(Value::Bool(pass))
}

fn eq(args: &Value) -> Result<Value> {
    let Value::Cons(pair) = args else {
        bail!("should give me a list");
    };

    let Value::Cons(pair2) = &pair.1 else {
        bail!("should give me a list");
    };

    if let Value::Cons(_) = pair2.1 {
        return Err(BuiltinError::BadEqArgCount.into());
    }

    match (&pair.0, &pair2.0) {
        (Value::Num(last), Value::Num(penu)) => Ok(Value::Bool(penu == last)),
        (Value::Str(last), Value::Str(penu))
        | (Value::Symbol(last), Value::Str(penu))
        | (Value::Str(last), Value::Symbol(penu))
        | (Value::Symbol(last), Value::Symbol(penu)) => Ok(Value::Bool(penu == last)),
        _ => Err(BuiltinError::BadEqArgTypes.into()),
    }
}

fn modulo(args: &Value) -> Result<Value> {
    let Value::Cons(pair) = args else {
        bail!("should give me a list");
    };

    let Value::Cons(pair2) = &pair.1 else {
        bail!("should give me a list");
    };

    if let Value::Cons(_) = pair2.1 {
        return Err(BuiltinError::BadModArgCount.into());
    }

    match (&pair.0, &pair2.0) {
        (Value::Num(last), Value::Num(penu)) => Ok(Value::Num(penu % last)),
        _ => Err(BuiltinError::BadModArgTypes.into()),
    }
}

fn not(args: &Value) -> Result<Value> {
    let Value::Cons(pair) = args else {
        bail!("should give me a list");
    };

    if let Value::Cons(_) = pair.1 {
        return Err(BuiltinError::WrongNotArgCount.into());
    }

    Ok(Value::Bool(!pair.0.truthy()))
}

fn cons(args: &Value) -> Result<Value> {
    let Value::Cons(args) = args else {
        return Err(BuiltinError::WrongConsArgCount.into());
    };

    let head = &args.0;

    let Value::Cons(tail_pair) = &args.1 else {
        return Err(BuiltinError::WrongConsArgCount.into());
    };

    if let Value::Cons(_) = tail_pair.1 {
        return Err(BuiltinError::WrongConsArgCount.into());
    }

    let tail = &tail_pair.0;

    // TODO(ajone239): make sure this clone is ok
    Ok(Value::Cons(Rc::new((head.clone(), tail.clone()))))
}

fn car(args: &Value) -> Result<Value> {
    let Value::Cons(args) = args else {
        bail!("should give me a list");
    };

    if let Value::Cons(_) = &args.1 {
        return Err(BuiltinError::WrongCarArgCount.into());
    };

    let pair = match &args.0 {
        Value::Nil => return Err(BuiltinError::CarOnEmptyList.into()),
        Value::Cons(pair) => pair,
        _ => return Err(BuiltinError::WrongCarArgType.into()),
    };

    if matches!(pair.0, Value::Nil) {
        return Err(BuiltinError::CarOnEmptyList.into());
    }

    Ok(pair.0.clone())
}

fn cdr(args: &Value) -> Result<Value> {
    let Value::Cons(args) = args else {
        bail!("should give me a list");
    };

    if let Value::Cons(_) = &args.1 {
        return Err(BuiltinError::WrongCdrArgCount.into());
    };

    let pair = match &args.0 {
        Value::Nil => return Err(BuiltinError::CdrOnEmptyList.into()),
        Value::Cons(pair) => pair,
        _ => return Err(BuiltinError::WrongCdrArgType.into()),
    };

    if matches!(pair.0, Value::Nil) {
        return Err(BuiltinError::CdrOnEmptyList.into());
    }

    Ok(pair.1.clone())
}

fn list(args: &Value) -> Result<Value> {
    Ok(args.clone())
}

fn print(args: &Value) -> Result<Value> {
    let out = args
        .to_vec()
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    println!("{}", out);

    Ok(Value::Nil)
}

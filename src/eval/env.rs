use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;
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
}

impl Default for Env {
    fn default() -> Self {
        let mut env = HashMap::new();

        let bins: &[(&str, Builtin)] = &[
            ("+", add),
            ("*", mul),
            ("-", min),
            ("/", div),
            ("<", lt),
            ("not", not),
            ("cons", cons),
            ("car", car),
            ("cdr", cdr),
            ("list", list),
        ];

        for (name, f) in bins {
            env.insert(name.to_string(), tobi(*f, name));
        }

        Self { env, parent: None }
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum BuiltinError {
    #[error("Not: Expected 1 argument got {0}.")]
    WrongNotArgCount(usize),
    #[error("Cons: Expected 2 arguments got {0}.")]
    WrongConsArgCount(usize),
    #[error("Car: Expected 1 argument got {0}.")]
    WrongCarArgCount(usize),
    #[error("Car: Must be applied to a list.")]
    WrongCarArgType,
    #[error("Cdr: Expected 1 argument got {0}.")]
    WrongCdrArgCount(usize),
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
    #[error("LessThan: Expected 1 argument got {0}")]
    BadLtArgCount(usize),
    #[error("Div: initial argument required.")]
    NoInitforDiv,
    #[error("Car: Cannot be applied to an empty list.")]
    CarOnEmptyList,
}

fn tobi(f: Builtin, name: &str) -> Value {
    Value::Builtin(BuiltinFn(f), name.to_owned())
}

fn args_to_num(args: &[Value]) -> impl Iterator<Item = Result<&f64>> {
    args.iter().map(move |v| match v {
        Value::Num(n) => Ok(n),
        _ => Err(BuiltinError::ExpectedNumArg.into()),
    })
}

pub fn add(args: &[Value]) -> Result<Value> {
    let args = args_to_num(args)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter();

    let num = args.fold(0.0, |acc, x| acc + x);
    Ok(Value::Num(num))
}

pub fn min(args: &[Value]) -> Result<Value> {
    let mut args = args_to_num(args)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter();

    let Some(init) = args.next() else {
        return Err(BuiltinError::NoInitforMinus.into());
    };

    let num = args.fold(*init, |acc, x| acc - x);
    Ok(Value::Num(num))
}

pub fn mul(args: &[Value]) -> Result<Value> {
    let args = args_to_num(args)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter();
    let num = args.fold(1.0, |acc, x| acc * x);
    Ok(Value::Num(num))
}

pub fn div(args: &[Value]) -> Result<Value> {
    let mut args = args_to_num(args)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter();

    let Some(init) = args.next() else {
        return Err(BuiltinError::NoInitforDiv.into());
    };

    let num = args.fold(*init, |acc, x| acc / x);
    Ok(Value::Num(num))
}

pub fn lt(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(BuiltinError::BadLtArgCount(args.len()).into());
    }

    match (&args[args.len() - 1], &args[args.len() - 2]) {
        (Value::Num(last), Value::Num(penu)) => Ok(Value::Bool(penu < last)),
        _ => Err(BuiltinError::BadLtArgTypes.into()),
    }
}

pub fn not(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(BuiltinError::WrongNotArgCount(args.len()).into());
    }
    let val = &args[0];
    Ok(Value::Bool(!val.truthy()))
}

pub fn cons(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(BuiltinError::WrongConsArgCount(args.len()).into());
    }

    let head = args[0].clone();
    let tail = args[1].clone();

    Ok(Value::List(vec![head, tail]))
}

pub fn car(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(BuiltinError::WrongCarArgCount(args.len()).into());
    }

    let Value::List(pair) = &args[0] else {
        return Err(BuiltinError::WrongCarArgType.into());
    };

    if pair.is_empty() {
        return Err(BuiltinError::CarOnEmptyList.into());
    }

    Ok(pair.first().expect("car expected items in list").clone())
}

pub fn cdr(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(BuiltinError::WrongCdrArgCount(args.len()).into());
    }

    let Value::List(pair) = &args[0] else {
        return Err(BuiltinError::WrongCdrArgType.into());
    };

    if pair.is_empty() {
        return Err(BuiltinError::CdrOnEmptyList.into());
    }

    if pair.len() == 2 && matches!(&pair[1], Value::List(_)) {
        return Ok(pair[1].clone());
    }

    Ok(Value::List(pair[1..].to_vec()))
}

pub fn list(args: &[Value]) -> Result<Value> {
    Ok(Value::List(args.to_vec()))
}

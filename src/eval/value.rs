use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;

use crate::eval::env::Env;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Num(f64),
    Symbol(String),
    Form(Form),
    Str(String),
    // TODO(ajone239): move this to a ref when copies get expensive
    List(Vec<Value>),
    Progn(Vec<Value>),
    Builtin(BuiltinFn),
    Func {
        name: String,
        args: Vec<String>,
        body: Rc<Value>,
    },
    Lambda {
        args: Vec<String>,
        body: Rc<Value>,
        env: Rc<RefCell<Env>>,
    },
}

impl Value {
    pub fn truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(val) => *val,
            Value::Num(num) => num.ne(&0.0),
            Value::Str(s) => !s.is_empty(),
            Value::List(v) | Value::Progn(v) => !v.is_empty(),
            Value::Symbol(_)
            | Value::Form(_)
            | Value::Builtin(_)
            | Value::Func { .. }
            | Value::Lambda { .. } => true,
        }
    }
}

pub type Builtin = fn(&[Value]) -> Result<Value>;

#[derive(Debug, Clone, Copy)]
pub struct BuiltinFn(pub Builtin);

impl PartialEq for BuiltinFn {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Form {
    If,
    Quote,
    Define,
    Lambda,
}

impl Form {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "if" => Some(Self::If),
            "quote" | "'" => Some(Self::Quote),
            "define" | "def" => Some(Self::Define),
            "lambda" | "lamda" | ".\\" => Some(Self::Lambda),
            _ => None,
        }
    }
}

use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::rc::Rc;

use anyhow::Result;

use crate::eval::env::Env;

#[derive(Clone, PartialEq)]
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
    Builtin(BuiltinFn, String),
    Macro {
        name: String,
        args: Vec<String>,
        body: Rc<Value>,
    },
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

#[derive(Clone, PartialEq)]
pub struct Lambda {}

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
            | Value::Builtin(_, _)
            | Value::Func { .. }
            | Value::Macro { .. }
            | Value::Lambda { .. } => true,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            Value::Num(n) => write!(f, "{}", n),
            Value::Symbol(s) => write!(f, ":{}", s),
            Value::Form(form) => write!(f, "{:?}", form),
            Value::Str(s) => write!(f, "{}", s),
            Value::List(values) => {
                let nice_list = values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" ");
                write!(f, "'({})", nice_list)
            }
            Value::Progn(values) => {
                let nice_list = values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" ");
                write!(f, "'(progn {})", nice_list)
            }
            Value::Builtin(_, name) => write!(f, "built-in: {} (...) {{...}}", name),
            Value::Func {
                name,
                args,
                body: _,
            } => write!(f, "func: {} ({}) {{...}}", name, args.join(" ")),
            Value::Macro {
                name,
                args,
                body: _,
            } => write!(f, "macro: {} ({}) {{...}}", name, args.join(" ")),
            Value::Lambda {
                args,
                body: _,
                env: _,
            } => write!(f, "lambda: ({}) {{...}}", args.join(" ")),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "Nil"),
            Self::Bool(arg0) => f.debug_tuple("Bool").field(arg0).finish(),
            Self::Num(arg0) => f.debug_tuple("Num").field(arg0).finish(),
            Self::Symbol(arg0) => f.debug_tuple("Symbol").field(arg0).finish(),
            Self::Form(arg0) => f.debug_tuple("Form").field(arg0).finish(),
            Self::Str(arg0) => f.debug_tuple("Str").field(arg0).finish(),
            Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
            Self::Progn(arg0) => f.debug_tuple("Progn").field(arg0).finish(),
            Self::Builtin(arg0, arg1) => f.debug_tuple("Builtin").field(arg0).field(arg1).finish(),
            Self::Macro { name, args, body } => f
                .debug_struct("Macro")
                .field("name", name)
                .field("args", args)
                .field("body", body)
                .finish(),
            Self::Func { name, args, body } => f
                .debug_struct("Func")
                .field("name", name)
                .field("args", args)
                .field("body", body)
                .finish(),
            Value::Lambda { args, body, env: _ } => f
                .debug_struct("Lambda")
                .field("args", args)
                .field("body", body)
                .field("env", &"{...}")
                .finish(),
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
    Eval,
    Require,
    Quote,
    QuasiQuote,
    UnQuote,
    Define,
    DefineMacro,
    Lambda,
    Progn,
}

impl Form {
    pub fn from_str(s: &str) -> Option<Self> {
        // TODO(ajone239): make weird symbols for all these
        match s {
            "if" => Some(Self::If),
            "require" => Some(Self::Require),
            "eval" => Some(Self::Eval),
            "progn" => Some(Self::Progn),
            "quote" | "'" => Some(Self::Quote),
            "quasiquote" | "`" => Some(Self::QuasiQuote),
            "unquote" | "," => Some(Self::UnQuote),
            "define" | "def" => Some(Self::Define),
            "defmacro" | "defm" => Some(Self::DefineMacro),
            "lambda" | "lamda" | ".\\" => Some(Self::Lambda),
            _ => None,
        }
    }
}

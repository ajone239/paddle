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
    Cons(Rc<(Value, Value)>),
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

impl Value {
    pub fn truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(val) => *val,
            Value::Num(num) => num.ne(&0.0),
            Value::Str(s) => !s.is_empty(),
            Value::Progn(v) => !v.is_empty(),
            Value::Cons(pair) => !matches!(pair.0, Self::Nil),
            Value::Symbol(_)
            | Value::Form(_)
            | Value::Builtin(_, _)
            | Value::Func { .. }
            | Value::Macro { .. }
            | Value::Lambda { .. } => true,
        }
    }

    pub fn to_cons_list(list: Vec<Self>) -> Self {
        let mut vals = list.into_iter().rev();

        let mut rv = Value::Nil;

        while let Some(val) = vals.next() {
            rv = Value::Cons(Rc::new((val, rv)));
        }

        rv
    }

    pub fn to_cons_iter(&self) -> ConsIter<'_> {
        ConsIter::new(self)
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
            Value::Cons(pair) => {
                let first = &pair.0;
                let mut second = &pair.1;

                if matches!(second, Value::Nil) {
                    return write!(f, "'({})", first);
                }

                let mut vals = vec![first.to_string()];

                while let Value::Cons(next_pair) = second {
                    let first = &next_pair.0;
                    vals.push(first.to_string());
                    second = &next_pair.1;
                }
                if !matches!(second, Self::Nil) {
                    vals.push(second.to_string());
                }

                let nice_list = vals.join(" ");
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

impl FromIterator<Value> for Value {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Value::to_cons_list(iter.into_iter().collect())
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "Nil"),
            Value::Bool(arg0) => f.debug_tuple("Bool").field(arg0).finish(),
            Value::Num(arg0) => f.debug_tuple("Num").field(arg0).finish(),
            Value::Symbol(arg0) => f.debug_tuple("Symbol").field(arg0).finish(),
            Value::Form(arg0) => f.debug_tuple("Form").field(arg0).finish(),
            Value::Str(arg0) => f.debug_tuple("Str").field(arg0).finish(),
            Value::Cons(arg0) => f.debug_tuple("Cons").field(arg0).finish(),
            Value::Progn(arg0) => f.debug_tuple("Progn").field(arg0).finish(),
            Value::Builtin(arg0, arg1) => f.debug_tuple("Builtin").field(arg0).field(arg1).finish(),
            Value::Macro { name, args, body } => f
                .debug_struct("Macro")
                .field("name", name)
                .field("args", args)
                .field("body", body)
                .finish(),
            Value::Func { name, args, body } => f
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

pub type Builtin = fn(&Value) -> Result<Value>;

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

pub struct ConsIter<'a> {
    current: &'a Value,
}

impl<'a> ConsIter<'a> {
    pub fn new(current: &'a Value) -> Self {
        Self { current }
    }
    pub fn len(self) -> usize {
        let mut len = 0;
        for _ in self {
            len += 1;
        }
        len
    }
}

impl<'a> Iterator for ConsIter<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        let Value::Cons(pair) = self.current else {
            return None;
        };

        self.current = &pair.1;

        Some(&pair.0)
    }
}

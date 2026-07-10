use std::cell::RefCell;
use std::default;
use std::fmt::{Debug, Display};
use std::rc::Rc;

use anyhow::{Result, bail};

use crate::eval::env::Env;
use crate::lexer::Span;

#[derive(Clone, PartialEq)]
pub enum Value {
    NoPrint,
    Nil(Span),
    Bool(bool, Span),
    Char(u8, Span),
    Num(f64, Span),
    Symbol(Rc<str>, Span),
    Form(Form, Span),
    Str(Rc<str>, Span),
    Cons(Rc<(Value, Value)>, Span),
    Builtin(BuiltinFn, String, Span),
    Macro {
        name: String,
        args: Vec<String>,
        body: Rc<Value>,
        span: Span,
    },
    Func {
        name: String,
        args: Vec<String>,
        body: Rc<Value>,
        env: Rc<RefCell<Env>>,
        span: Span,
    },
    Lambda {
        args: Vec<String>,
        body: Rc<Value>,
        env: Rc<RefCell<Env>>,
        span: Span,
    },
}

impl Value {
    pub fn truthy(&self) -> bool {
        match self {
            Value::NoPrint => false,
            Value::Nil(_) => false,
            Value::Bool(val, _) => *val,
            Value::Num(num, _) => num.ne(&0.0),
            Value::Char(byte, _) => *byte != 0,
            Value::Str(s, _) => !s.is_empty(),
            Value::Cons(_, _)
            | Value::Symbol(_, _)
            | Value::Form(_, _)
            | Value::Builtin(_, _, _)
            | Value::Func { .. }
            | Value::Macro { .. }
            | Value::Lambda { .. } => true,
        }
    }

    pub fn to_cons_list(list: Vec<Self>) -> Self {
        let mut rv = Value::Nil(Span::default());
        for val in list.into_iter().rev() {
            let span = val.get_span();
            rv = Value::Cons(Rc::new((val, rv)), span);
        }
        rv
    }

    pub fn get_span(&self) -> Span {
        match self {
            Value::NoPrint => Span::default(),
            Value::Nil(s)
            | Value::Bool(_, s)
            | Value::Num(_, s)
            | Value::Char(_, s)
            | Value::Str(_, s)
            | Value::Cons(_, s)
            | Value::Symbol(_, s)
            | Value::Form(_, s)
            | Value::Builtin(_, _, s)
            | Value::Func { span: s, .. }
            | Value::Macro { span: s, .. }
            | Value::Lambda { span: s, .. } => s.clone(),
        }
    }

    pub fn to_cons_iter(&self) -> ConsIter<'_> {
        ConsIter::new(self)
    }

    pub fn splice(&self, other: Self) -> Result<Self> {
        if matches!(self, Self::Nil(_)) {
            return Ok(other);
        }
        if !matches!(self, Self::Cons(_, _)) {
            bail!("has to be a list")
        }

        let vals: Vec<&Value> = self.to_cons_iter().collect();

        let mut rv = other;

        for val in vals.into_iter().rev() {
            let span = val.get_span();
            rv = Self::Cons(Rc::new((val.clone(), rv)), span);
        }

        Ok(rv)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::NoPrint => Ok(()),
            Value::Nil(_) => write!(f, "nil"),
            Value::Bool(b, _) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            Value::Num(n, _) => write!(f, "{}", n),
            Value::Char(b, _) => write!(f, "'{}'", char::from(*b)),
            Value::Symbol(s, _) => write!(f, ":{}", s),
            Value::Form(form, _) => write!(f, "{:?}", form),
            Value::Str(s, _) => write!(f, "{}", s),
            Value::Cons(pair, _) => {
                let first = &pair.0;
                let mut second = &pair.1;

                if matches!(second, Value::Nil(_)) {
                    return write!(f, "'({})", first);
                }

                let mut vals = vec![first.to_string()];

                while let Value::Cons(next_pair, _) = second {
                    let first = &next_pair.0;
                    vals.push(first.to_string());
                    second = &next_pair.1;
                }
                if !matches!(second, Self::Nil(_)) {
                    vals.push(second.to_string());
                }

                let nice_list = vals.join(" ");
                write!(f, "'({})", nice_list)
            }
            Value::Builtin(_, name, _) => write!(f, "built-in: {} (...) {{...}}", name),
            Value::Func { name, args, .. } => {
                write!(f, "func: {} ({}) {{...}}", name, args.join(" "))
            }
            Value::Macro { name, args, .. } => {
                write!(f, "macro: {} ({}) {{...}}", name, args.join(" "))
            }
            Value::Lambda { args, .. } => write!(f, "lambda: ({}) {{...}}", args.join(" ")),
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
            Self::NoPrint => write!(f, "NoPrint"),
            Self::Nil(arg0) => f.debug_tuple("Nil").field(arg0).finish(),
            Self::Bool(arg0, arg1) => f.debug_tuple("Bool").field(arg0).field(arg1).finish(),
            Self::Char(arg0, arg1) => f.debug_tuple("Char").field(arg0).field(arg1).finish(),
            Self::Num(arg0, arg1) => f.debug_tuple("Num").field(arg0).field(arg1).finish(),
            Self::Symbol(arg0, arg1) => f.debug_tuple("Symbol").field(arg0).field(arg1).finish(),
            Self::Form(arg0, arg1) => f.debug_tuple("Form").field(arg0).field(arg1).finish(),
            Self::Str(arg0, arg1) => f.debug_tuple("Str").field(arg0).field(arg1).finish(),
            Self::Cons(arg0, arg1) => f.debug_tuple("Cons").field(arg0).field(arg1).finish(),
            Self::Builtin(arg0, arg1, arg2) => f
                .debug_tuple("Builtin")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::Macro {
                name,
                args,
                body,
                span,
            } => f
                .debug_struct("Macro")
                .field("name", name)
                .field("args", args)
                .field("body", body)
                .field("span", span)
                .field("env", &"{...}")
                .finish(),
            Value::Func {
                name,
                args,
                body,
                span,
                env: _,
            } => f
                .debug_struct("Func")
                .field("name", name)
                .field("args", args)
                .field("body", body)
                .field("span", span)
                .field("env", &"{...}")
                .finish(),
            Value::Lambda {
                args,
                body,
                span,
                env: _,
            } => f
                .debug_struct("Lambda")
                .field("args", args)
                .field("body", body)
                .field("span", span)
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
    UnQuoteSplicing,
    Define,
    DefineMacro,
    Lambda,
    Progn,
    SetBang,
}

impl Form {
    pub fn try_parse(s: &str) -> Option<Self> {
        // TODO(ajone239): make weird symbols for all these
        match s {
            "if" => Some(Self::If),
            "require" => Some(Self::Require),
            "eval" => Some(Self::Eval),
            "progn" => Some(Self::Progn),
            "set!" => Some(Self::SetBang),
            "quote" | "'" => Some(Self::Quote),
            "quasiquote" | "`" => Some(Self::QuasiQuote),
            "unquotesplicing" | "@" => Some(Self::UnQuoteSplicing),
            "unquote" | "," => Some(Self::UnQuote),
            "define" | "def" => Some(Self::Define),
            "defmacro" | "defm" => Some(Self::DefineMacro),
            "lambda" | "lamda" | ".\\" => Some(Self::Lambda),
            _ => None,
        }
    }
}

pub struct ConsIter<'a> {
    current: Option<&'a Value>,
}

impl<'a> ConsIter<'a> {
    pub fn new(current: &'a Value) -> Self {
        Self {
            current: Some(current),
        }
    }

    pub fn into_cons_list(&mut self) -> Value {
        let Some(cons) = self.current else {
            return Value::Nil(Span::default());
        };

        self.current = None;
        cons.clone()
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.current, None) || !matches!(self.current, Some(Value::Cons(_, _)))
    }
}

impl<'a> Iterator for ConsIter<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(Value::Cons(pair, _)) = self.current else {
            return None;
        };

        self.current = Some(&pair.1);

        Some(&pair.0)
    }
}

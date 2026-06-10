use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, ErrorKind, Read},
    rc::Rc,
};

use anyhow::{Context, Result, bail};
use thiserror::Error;

use crate::{
    cursor::process,
    eval::value::{Builtin, BuiltinFn, Value},
};

static STD_LIB: &str = include_str!("../../../examples/base.pd");
static STD_MAC: &str = include_str!("../../../examples/macros.pd");

#[derive(Debug, PartialEq)]
pub struct Env {
    env: HashMap<String, Value>,
    builtin: Rc<HashMap<String, Value>>,
    parent: Option<Rc<RefCell<Self>>>,
}

impl Env {
    pub fn new_child(parent: Rc<RefCell<Self>>) -> Self {
        let builtin = parent.borrow().builtin.clone();

        Self {
            env: HashMap::new(),
            builtin,
            parent: Some(parent),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.env.insert(name.to_owned(), value);
    }

    pub fn set_bang(&mut self, name: &str, value: Value) -> Result<()> {
        if self.env.get(name).is_some() {
            self.env.insert(name.to_string(), value);
            return Ok(());
        }

        let mut parent = self.parent.clone();
        while let Some(penv) = parent {
            let mut benv = penv.borrow_mut();
            if benv.env.get(name).is_some() {
                benv.env.insert(name.to_string(), value);
                return Ok(());
            }
            parent = benv.parent.clone();
        }

        bail!(
            "Can only set! an existing variable: {} isn't in scope",
            name
        );
    }

    pub fn resolve(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.builtin.get(name) {
            return Some(val.clone());
        }

        if let Some(val) = self.env.get(name) {
            return Some(val.clone());
        }

        let mut parent = self.parent.clone();
        while let Some(penv) = parent {
            if let Some(val) = penv.borrow().env.get(name) {
                return Some(val.clone());
            }
            parent = penv.borrow().parent.clone();
        }

        None
    }

    pub fn with_stdlib() -> Result<Rc<RefCell<Self>>> {
        let env = Rc::new(RefCell::new(Env::default()));

        process(STD_LIB, env.clone()).context("failed to parse the std lib")?;
        process(STD_MAC, env.clone()).context("failed to parse the std lib macros")?;

        Ok(env)
    }

    pub fn small_dump(&self) {
        for (k, v) in self
            .env
            .iter()
            .filter(|(_, v)| !matches!(v, Value::Builtin(..)))
        {
            println!("{}: {}", k, v)
        }
        println!();
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
        let mut benv = HashMap::new();

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
            ("atom?", is_atom),
            ("number?", is_number),
            ("symbol?", is_symbol),
            ("string?", is_string),
            ("char?", is_char),
            ("char", make_char),
            ("null?", is_null),
            ("pair?", is_pair),
            ("getchar", getchar),
            ("getline", getline),
        ];

        for (name, f) in bins {
            benv.insert(name.to_string(), tobi(*f, name));
        }

        Self {
            env: HashMap::new(),
            builtin: Rc::new(benv),
            parent: None,
        }
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
    #[error("Eq: Expected 2 arguments")]
    BadEqArgCount,
}

fn tobi(f: Builtin, name: &str) -> Value {
    Value::Builtin(BuiltinFn(f), name.to_owned())
}

fn add(args: &Value) -> Result<Value> {
    if !matches!(args, Value::Cons(_) | Value::Nil) {
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
    if !matches!(args, Value::Cons(_) | Value::Nil) {
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
        return Err(BuiltinError::NoInitforDiv.into());
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
    let mut pass = false;

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
        (Value::Nil, Value::Nil) => Ok(Value::Bool(true)),
        (_, Value::Nil) | (Value::Nil, _) => Ok(Value::Bool(false)),
        (Value::Char(last), Value::Char(penu)) => Ok(Value::Bool(penu == last)),
        (Value::Char(byte), Value::Str(s))
        | (Value::Char(byte), Value::Symbol(s))
        | (Value::Str(s), Value::Char(byte))
        | (Value::Symbol(s), Value::Char(byte)) => {
            if s.len() != 1 {
                return Ok(Value::Bool(false));
            }

            let s = s.bytes().next().unwrap();
            Ok(Value::Bool(s == *byte))
        }
        (Value::Str(last), Value::Str(penu))
        | (Value::Symbol(last), Value::Str(penu))
        | (Value::Str(last), Value::Symbol(penu))
        | (Value::Symbol(last), Value::Symbol(penu)) => Ok(Value::Bool(penu == last)),
        _ => Ok(Value::Bool(false)),
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

    match (&pair2.0, &pair.0) {
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

    Ok(pair.1.clone())
}

fn list(args: &Value) -> Result<Value> {
    Ok(args.clone())
}

fn print(args: &Value) -> Result<Value> {
    let out = args
        .to_cons_iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    println!("{}", out);

    Ok(Value::Nil)
}

fn is_number(args: &Value) -> Result<Value> {
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

fn is_symbol(args: &Value) -> Result<Value> {
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

fn is_char(args: &Value) -> Result<Value> {
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

fn is_string(args: &Value) -> Result<Value> {
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

fn is_atom(args: &Value) -> Result<Value> {
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

fn is_null(args: &Value) -> Result<Value> {
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

fn is_pair(args: &Value) -> Result<Value> {
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

fn getchar(_args: &Value) -> Result<Value> {
    let mut buf = [0u8; 1];

    let res = io::stdin().read_exact(&mut buf);

    match res {
        Ok(_) => Ok(Value::Char(buf[0])),
        Err(err) if err.kind() == ErrorKind::UnexpectedEof => Ok(Value::Cons(Rc::new((
            Value::Symbol("err".into()),
            Value::Str("EOF".into()),
        )))),
        Err(err) => Err(err.into()),
    }
}

fn make_char(args: &Value) -> Result<Value> {
    let Value::Cons(args) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_) = &args.1 {
        bail!("only one arg");
    };

    let byte = match args.0 {
        Value::Symbol(ref args) | Value::Str(ref args) if args.len() == 1 => {
            args.bytes().next().unwrap()
        }
        Value::Num(byte) if byte > 0.0 && byte < 256.0 => byte as u8,
        _ => bail!("char takes num, sym, or str"),
    };

    Ok(Value::Char(byte))
}

fn getline(_args: &Value) -> Result<Value> {
    let mut buf = String::new();
    let res = io::stdin().read_line(&mut buf);

    match res {
        Ok(_) => Ok(Value::Str(buf.trim().into())),
        Err(err) => Err(err.into()),
    }
}

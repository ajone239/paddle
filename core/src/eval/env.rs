use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::{Context, Result, bail};
use thiserror::Error;

use crate::{
    cursor::process,
    eval::{
        builtins,
        value::{Builtin, BuiltinFn, Value},
    },
};

static STD_LIB: &str = include_str!("../../../examples/base.pd");
static STD_MAC: &str = include_str!("../../../examples/macros.pd");

#[derive(Debug, PartialEq)]
pub struct Env {
    env: HashMap<String, Value>,
    builtin: Rc<HashMap<String, Value>>,
    parent: Option<Rc<RefCell<Self>>>,
}

impl Default for Env {
    fn default() -> Self {
        let mut benv = HashMap::new();

        let bins: &[(&str, Builtin)] = &[
            ("+", builtins::math::add),
            ("*", builtins::math::mul),
            ("-", builtins::math::min),
            ("/", builtins::math::div),
            ("<", builtins::math::lt),
            ("%", builtins::math::modulo),
            ("=", builtins::boolean::eq),
            ("&&", builtins::boolean::and),
            ("||", builtins::boolean::val_or),
            ("not", builtins::boolean::not),
            ("cons", builtins::list::cons),
            ("car", builtins::list::car),
            ("cdr", builtins::list::cdr),
            ("list", builtins::list::list),
            // ("append", builtins::list::append),
            ("atom?", builtins::predicate::is_atom),
            ("number?", builtins::predicate::is_number),
            ("symbol?", builtins::predicate::is_symbol),
            ("string?", builtins::predicate::is_string),
            ("char?", builtins::predicate::is_char),
            ("null?", builtins::predicate::is_null),
            ("pair?", builtins::predicate::is_pair),
            ("print", builtins::inandout::print),
            ("getchar", builtins::inandout::getchar),
            ("getline", builtins::inandout::getline),
            ("char", builtins::string::make_char),
            ("string-length", builtins::string::string_length),
            ("string-ref", builtins::string::string_ref),
            ("substring", builtins::string::substring),
            ("string-append", builtins::string::string_append),
            ("string->list", builtins::string::string_list),
            ("string->num", builtins::string::string_num),
            ("list->string", builtins::string::list_string),
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

fn tobi(f: Builtin, name: &str) -> Value {
    Value::Builtin(BuiltinFn(f), name.to_owned())
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
        if self.env.contains_key(name) {
            self.env.insert(name.to_string(), value);
            return Ok(());
        }

        let mut parent = self.parent.clone();
        while let Some(penv) = parent {
            let mut benv = penv.borrow_mut();
            if benv.env.contains_key(name) {
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

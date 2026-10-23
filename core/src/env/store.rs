use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::{Context, Result, bail};

use crate::{
    cursor::process_named_bytes,
    span::Span,
    value::{Builtin, BuiltinFn, Value},
};

use super::builtins;

static STD_LIB_FILE: &str = "base.pd";
static STD_LIB: &str = include_str!("../../../examples/base.pd");
static STD_MAC_FILE: &str = "macros.pd";
static STD_MAC: &str = include_str!("../../../examples/macros.pd");

#[derive(Debug, PartialEq)]
pub struct Env {
    pub current_file: String,
    preexpand_macros: bool,
    env: HashMap<String, Value>,
    builtin: Rc<HashMap<String, Value>>,
    parent: Option<Rc<RefCell<Self>>>,
}

impl Default for Env {
    fn default() -> Self {
        Self::new(true)
    }
}

fn tobi(f: Builtin) -> Value {
    Value::Builtin(BuiltinFn(f), Span::default())
}

impl Env {
    pub fn new(preexpand_macros: bool) -> Self {
        let mut benv = HashMap::new();

        let bins: &[(&str, Builtin)] = &[
            ("+", builtins::math::add),
            ("*", builtins::math::mul),
            ("-", builtins::math::min),
            ("/", builtins::math::div),
            ("//", builtins::math::intdiv),
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
            ("append", builtins::list::append),
            ("atom?", builtins::predicate::is_atom),
            ("bool?", builtins::predicate::is_bool),
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
            ("char->num", builtins::string::char_ord),
            ("string-length", builtins::string::string_length),
            ("string-ref", builtins::string::string_ref),
            ("substring", builtins::string::substring),
            ("string-append", builtins::string::string_append),
            ("string->list", builtins::string::string_list),
            ("string->num", builtins::string::string_num),
            ("string->symbol", builtins::string::string_symbol),
            ("list->string", builtins::string::list_string),
        ];

        for (name, f) in bins {
            benv.insert(name.to_string(), tobi(*f));
        }

        Self {
            current_file: "__main__".to_string(),
            env: HashMap::new(),
            builtin: Rc::new(benv),
            preexpand_macros,
            parent: None,
        }
    }

    pub fn new_child(parent: Rc<RefCell<Self>>) -> Self {
        let builtin = parent.borrow().builtin.clone();
        let current_file = parent.borrow().current_file.to_owned();
        let preexpand_macros = parent.borrow().preexpand_macros;

        Self {
            current_file,
            env: HashMap::new(),
            preexpand_macros,
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

    pub fn preexpand_macros(&self) -> bool {
        self.preexpand_macros
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

    pub fn with_stdlib(preexpand_macros: bool) -> Result<Rc<RefCell<Self>>> {
        let env = Rc::new(RefCell::new(Env::new(preexpand_macros)));

        process_named_bytes(STD_MAC_FILE.into(), STD_MAC, env.clone())
            .context("failed to parse the std lib macros")?;
        process_named_bytes(STD_LIB_FILE.into(), STD_LIB, env.clone())
            .context("failed to parse the std lib")?;

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

        println!("Built-Ins:");
        for (k, v) in self.builtin.iter() {
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

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::eval::{Builtin, BuiltinFn, Value};

#[derive(Debug)]
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
        // walk resolve here
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

        env.insert("+".to_string(), tobi(add));
        env.insert("*".to_string(), tobi(mul));
        env.insert("-".to_string(), tobi(min));
        env.insert("/".to_string(), tobi(div));
        env.insert("<".to_string(), tobi(lt));

        env.insert("cons".to_string(), tobi(cons));
        env.insert("car".to_string(), tobi(car));
        env.insert("cdr".to_string(), tobi(cdr));

        Self { env, parent: None }
    }
}

fn tobi(f: Builtin) -> Value {
    Value::Builtin(BuiltinFn(f))
}

fn args_to_num(args: &[Value]) -> impl Iterator<Item = &f64> {
    args.iter().map(move |v| match v {
        Value::Num(n) => n,
        _ => todo!("bad call args: {:?}", args),
    })
}

pub fn add(args: &[Value]) -> Value {
    let args = args_to_num(args);
    let num = args.fold(0.0, |acc, x| acc + x);
    Value::Num(num)
}

pub fn min(args: &[Value]) -> Value {
    let mut args = args_to_num(args);
    let init = *args.next().unwrap();
    let num = args.fold(init, |acc, x| acc - x);
    Value::Num(num)
}

pub fn mul(args: &[Value]) -> Value {
    let args = args_to_num(args);
    let num = args.fold(1.0, |acc, x| acc * x);
    Value::Num(num)
}

pub fn div(args: &[Value]) -> Value {
    let mut args = args_to_num(args);
    let init = *args.next().unwrap();
    let num = args.fold(init, |acc, x| acc / x);
    Value::Num(num)
}

pub fn lt(args: &[Value]) -> Value {
    let Value::Num(last) = args[args.len() - 1] else {
        panic!("ahhh")
    };
    let Value::Num(penu) = args[args.len() - 2] else {
        panic!("ahhh")
    };
    Value::Bool(penu < last)
}

pub fn cons(args: &[Value]) -> Value {
    if args.len() != 2 {
        panic!("cons takes 2 args");
    }

    let head = args[0].clone();
    let tail = args[1].clone();

    Value::List(vec![head, tail])
}

pub fn car(args: &[Value]) -> Value {
    if args.len() != 1 {
        panic!("car takes 1 args");
    }

    let Value::List(pair) = &args[0] else {
        panic!("car expected list");
    };

    pair.first().expect("car expected items in list").clone()
}

pub fn cdr(args: &[Value]) -> Value {
    if args.len() != 1 {
        panic!("cdr takes 1 args");
    }

    let Value::List(pair) = &args[0] else {
        panic!("cdr expected list");
    };

    if pair.is_empty() {
        panic!("cdr expected items in list");
    }

    if pair.len() == 2 && matches!(&pair[1], Value::List(_)) {
        return pair[1].clone();
    }

    Value::List(pair[1..].to_vec())
}

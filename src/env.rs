use std::collections::HashMap;

use crate::eval::Value;

#[derive(Debug)]
pub struct Env {
    env: HashMap<String, Value>,
}

impl Env {
    pub fn default() -> Self {
        let mut env = HashMap::new();

        env.insert("+".to_string(), Value::Builtin(add));
        env.insert("*".to_string(), Value::Builtin(mul));
        env.insert("-".to_string(), Value::Builtin(min));
        env.insert("/".to_string(), Value::Builtin(div));
        env.insert("<".to_string(), Value::Builtin(lt));

        Self { env }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.env.insert(name.to_owned(), value);
    }

    pub fn resolve(&self, name: &str) -> Option<&Value> {
        self.env.get(name)
    }
}

fn args_to_num<'a>(args: &'a [Value]) -> impl Iterator<Item = &'a f64> {
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

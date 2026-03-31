use std::collections::HashMap;

use crate::{eval::Value, math};

#[derive(Debug)]
pub struct Env {
    env: HashMap<String, Value>,
}

impl Env {
    pub fn default() -> Self {
        let mut env = HashMap::new();

        env.insert("+".to_string(), Value::Builtin(math::add));
        env.insert("*".to_string(), Value::Builtin(math::mul));
        env.insert("-".to_string(), Value::Builtin(math::min));
        env.insert("/".to_string(), Value::Builtin(math::div));

        Self { env }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.env.insert(name.to_owned(), value);
    }

    pub fn resolve(&self, name: &str) -> Option<&Value> {
        self.env.get(name)
    }
}

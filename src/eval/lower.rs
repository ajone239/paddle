use std::rc::Rc;

use crate::eval::value::{Form, Value};
use crate::parser::Expr;

pub fn lower(ast: &Expr) -> Value {
    quote_eval(ast)
}

fn quote_eval(ast: &Expr) -> Value {
    match ast {
        Expr::Atom(atom, _) => classify(atom),
        Expr::List(list, _) => {
            let mut vals = list.iter().map(quote_eval).rev();

            let first = vals.next().unwrap_or(Value::Nil);
            let mut rv = Rc::new((first, Value::Nil));

            while let Some(val) = vals.next() {
                rv = Rc::new((val, Value::Cons(rv)));
            }

            Value::Cons(rv)
        }
    }
}

fn classify(atom: &str) -> Value {
    if let Ok(num) = atom.parse::<f64>() {
        return Value::Num(num);
    }

    if let Some(form) = Form::from_str(atom) {
        return Value::Form(form);
    }

    match atom {
        "nil" => Value::Nil,
        "#t" => Value::Bool(true),
        "#f" => Value::Bool(false),
        _ if atom.starts_with('"') && atom.ends_with('"') => Value::Str(
            atom.strip_prefix("\"")
                .unwrap()
                .strip_suffix("\"")
                .unwrap()
                .to_owned(),
        ),
        _ => Value::Symbol(atom.to_owned()),
    }
}

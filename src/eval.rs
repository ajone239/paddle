use core::panic;

use crate::parser::Expr;

#[derive(Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Num(f64),
    Symbol(String),
    Str(String),
    List(Vec<Value>),
}

pub fn eval<'a>(ast: &Expr<'a>) -> Value {
    match ast {
        Expr::Atom(atom, _) => resolve(atom),
        Expr::List(list, _) => apply(&list),
    }
}

fn apply<'a>(list: &[Expr<'a>]) -> Value {
    if list.is_empty() {
        return Value::Nil;
    }

    let vals: Vec<_> = list.iter().map(|e| eval(e)).collect();
    let func = &vals[0];
    let args = &vals[1..];

    call(func, args)
}

fn call<'a>(func: &Value, args: &[Value]) -> Value {
    let mut args = args.iter().map(|v| match v {
        Value::Num(n) => n,
        _ => todo!(),
    });

    let Value::Symbol(s) = func else {
        panic!("bad expr head")
    };

    let num = match s.as_str() {
        "+" => args.fold(0.0, |acc, x| acc + x),
        "*" => args.fold(1.0, |acc, x| acc * x),
        "-" => {
            let init = *args.next().unwrap();
            args.fold(init, |acc, x| acc - x)
        }
        "/" => {
            let init = *args.next().unwrap();
            args.fold(init, |acc, x| acc / x)
        }
        _ => panic!("operation not supported"),
    };
    Value::Num(num)
}

fn resolve<'a>(atom: &'a str) -> Value {
    if let Ok(num) = atom.parse::<f64>() {
        return Value::Num(num);
    }

    match atom {
        "nil" => Value::Nil,
        "#t" => Value::Bool(true),
        "#f" => Value::Bool(false),
        "+" | "*" | "-" | "/" => Value::Symbol(atom.to_owned()),
        _ => Value::Str(atom.to_owned()),
    }
}

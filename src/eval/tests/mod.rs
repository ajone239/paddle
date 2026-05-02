mod arithmetic;
mod atoms;
mod builtin_errors;
mod cadr;
mod car;
mod cdr;
mod cons;
mod empty;
mod env;
mod eval_errors;
mod evalenv;
mod evalif;
mod lambda;
mod list;
mod macros;
mod quote;
mod variadic;

use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::eval::value::Value;
use crate::lexer::lex;
use crate::parser::parse_expr;

fn eval_str(s: &str) -> Value {
    let env = Env::default();
    let tokens = lex(s);
    let (expr, _) = parse_expr(&tokens).unwrap();
    let expr = lower(&expr);
    eval(&expr, Rc::new(RefCell::new(env))).unwrap()
}

fn eval_str_env(exprs: &[&str]) -> Value {
    let env = Rc::new(RefCell::new(Env::default()));

    let mut last = None;

    for expr in exprs {
        let tokens = lex(expr);
        let (e, _) = parse_expr(&tokens).unwrap();
        let e = lower(&e);
        let val = eval(&e, env.clone());
        last = Some(val);
    }

    last.unwrap().unwrap()
}

fn num(n: f64) -> Value {
    Value::Num(n)
}

fn eval_err(s: &str) -> anyhow::Error {
    let env = Env::default();
    let tokens = lex(s);
    let (expr, _) = parse_expr(&tokens).unwrap();
    let expr = lower(&expr);
    eval(&expr, Rc::new(RefCell::new(env))).unwrap_err()
}

fn eval_env_err(exprs: &[&str]) -> anyhow::Error {
    let env = Rc::new(RefCell::new(Env::default()));
    for s in exprs {
        let tokens = lex(s);
        let (e, _) = parse_expr(&tokens).unwrap();
        let e = lower(&e);
        if let Err(err) = eval(&e, env.clone()) {
            return err;
        }
    }
    panic!("expected an error but all expressions succeeded");
}

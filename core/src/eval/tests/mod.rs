mod arithmetic;
mod atoms;
mod builtin_errors;
mod cadr;
mod car;
mod cdr;
mod char_tests;
mod cons;
mod empty;
mod env;
mod eq;
mod eval_errors;
mod lambda;
mod list;
mod macros;
mod predicates;
mod quote;
mod set_bang;
mod splice;
mod string_tests;
mod tco;
mod variadic;

use std::cell::RefCell;
use std::rc::Rc;

use super::*;
use crate::eval::value::Value;
use crate::lexer::{Span, lex};
use crate::parser::parse_expr;

// `Value` and `EvalError` both carry a `Span` now, and both derive `PartialEq`
// over it. Real spans from parsing never equal the `Span::default()` placeholder
// tests build expected values with, so every comparison helper normalizes spans
// away before handing values back to the tests.
pub(crate) fn strip_value_span(v: &Value) -> Value {
    match v {
        Value::NoPrint => Value::NoPrint,
        Value::Nil(_) => Value::Nil(Span::default()),
        Value::Bool(b, _) => Value::Bool(*b, Span::default()),
        Value::Char(c, _) => Value::Char(*c, Span::default()),
        Value::Num(n, _) => Value::Num(*n, Span::default()),
        Value::Symbol(s, _) => Value::Symbol(s.clone(), Span::default()),
        Value::Form(f, _) => Value::Form(*f, Span::default()),
        Value::Str(s, _) => Value::Str(s.clone(), Span::default()),
        Value::Cons(pair, _) => Value::Cons(
            Rc::new((strip_value_span(&pair.0), strip_value_span(&pair.1))),
            Span::default(),
        ),
        Value::Builtin(f, name, _) => Value::Builtin(*f, name.clone(), Span::default()),
        Value::Macro {
            name,
            args,
            body,
            span: _,
        } => Value::Macro {
            name: name.clone(),
            args: args.clone(),
            body: Rc::new(strip_value_span(body)),
            span: Span::default(),
        },
        Value::Func {
            name,
            args,
            body,
            env,
            span: _,
        } => Value::Func {
            name: name.clone(),
            args: args.clone(),
            body: Rc::new(strip_value_span(body)),
            env: env.clone(),
            span: Span::default(),
        },
        Value::Lambda {
            args,
            body,
            env,
            span: _,
        } => Value::Lambda {
            args: args.clone(),
            body: Rc::new(strip_value_span(body)),
            env: env.clone(),
            span: Span::default(),
        },
    }
}

fn strip_eval_error_span(e: EvalError) -> EvalError {
    match e {
        EvalError::BadRequireArgCount(n, _) => EvalError::BadRequireArgCount(n, Span::default()),
        EvalError::BadRequireArgs(_) => EvalError::BadRequireArgs(Span::default()),
        EvalError::BadSetBangArgs(_) => EvalError::BadSetBangArgs(Span::default()),
        EvalError::BadDefineArgs(_) => EvalError::BadDefineArgs(Span::default()),
        EvalError::BadIfArgs(_) => EvalError::BadIfArgs(Span::default()),
        EvalError::BadLambdaArgs(_) => EvalError::BadLambdaArgs(Span::default()),
        EvalError::BadLambdaArgsList(_) => EvalError::BadLambdaArgsList(Span::default()),
        EvalError::SymbolUndefined(s, _) => EvalError::SymbolUndefined(s, Span::default()),
        EvalError::BadDefineHead(_) => EvalError::BadDefineHead(Span::default()),
        EvalError::BadLambdaArgsListType(_) => EvalError::BadLambdaArgsListType(Span::default()),
        EvalError::BadFunctionArgCount(n, _) => EvalError::BadFunctionArgCount(n, Span::default()),
        EvalError::BadDefineFunctionHead(_) => EvalError::BadDefineFunctionHead(Span::default()),
        EvalError::BadDefineFunctionHeadTypes(_) => {
            EvalError::BadDefineFunctionHeadTypes(Span::default())
        }
        EvalError::EmptyPrognBody(_) => EvalError::EmptyPrognBody(Span::default()),
        EvalError::UnquoteOutsideQuasi(_) => EvalError::UnquoteOutsideQuasi(Span::default()),
        EvalError::VariadicArgsMustBeLast(_) => EvalError::VariadicArgsMustBeLast(Span::default()),
        EvalError::BadCallableArgs(f, _) => EvalError::BadCallableArgs(f, Span::default()),
        EvalError::BadCallableArgsListType(f, _) => {
            EvalError::BadCallableArgsListType(f, Span::default())
        }
        EvalError::BadCallableHead(f, _) => EvalError::BadCallableHead(f, Span::default()),
        EvalError::BadCallableBodyArgs(f, _) => EvalError::BadCallableBodyArgs(f, Span::default()),
    }
}

fn normalize_err(err: anyhow::Error) -> anyhow::Error {
    match err.downcast::<EvalError>() {
        Ok(e) => strip_eval_error_span(e).into(),
        Err(err) => err,
    }
}

fn eval_str(s: &str) -> Value {
    let env = Env::default();
    let tokens = lex(s);
    let (expr, _) = parse_expr(&tokens).unwrap();
    let expr = lower(&expr);
    let val = eval(expr, Rc::new(RefCell::new(env))).unwrap();
    strip_value_span(&val)
}

fn eval_str_env(exprs: &[&str]) -> Value {
    let env = Rc::new(RefCell::new(Env::default()));

    let mut last = None;

    for expr in exprs {
        let tokens = lex(expr);
        let (e, _) = parse_expr(&tokens).unwrap();
        let e = lower(&e);
        let val = eval(e, env.clone());
        last = Some(val);
    }

    strip_value_span(&last.unwrap().unwrap())
}

fn num(n: f64) -> Value {
    Value::Num(n, Span::default())
}

fn sym(s: &str) -> Value {
    Value::Symbol(s.into(), Span::default())
}

fn cons(head: Value, tail: Value) -> Value {
    Value::Cons(Rc::new((head, tail)), Span::default())
}

fn eval_err(s: &str) -> anyhow::Error {
    let env = Env::default();
    let tokens = lex(s);
    let (expr, _) = parse_expr(&tokens).unwrap();
    let expr = lower(&expr);
    let err = eval(expr, Rc::new(RefCell::new(env))).unwrap_err();
    normalize_err(err)
}

fn eval_env_err(exprs: &[&str]) -> anyhow::Error {
    let env = Rc::new(RefCell::new(Env::default()));
    for s in exprs {
        let tokens = lex(s);
        let (e, _) = parse_expr(&tokens).unwrap();
        let e = lower(&e);
        if let Err(err) = eval(e, env.clone()) {
            return normalize_err(err);
        }
    }
    panic!("expected an error but all expressions succeeded");
}

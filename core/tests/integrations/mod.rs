//! Integration tests for the paddle LISP interpreter.
//!
//! These tests exercise the full pipeline — lex → parse → lower → eval —
//! through the public `process` API, mirroring how the binary loads and
//! runs programs (stdlib first, then user code).

use std::{cell::RefCell, rc::Rc};

use paddle_core::{
    cursor::process,
    eval::{Env, EvalError, value::Value},
    lexer::Span,
};

static STD_LIB: &str = include_str!("../../../examples/base.pd");
static MACROS_LIB: &str = include_str!("../../../examples/macros.pd");
static FACT_PROGRAM: &str = include_str!("../../../examples/fact.pd");
static IMPORT_PROGRAM: &str = include_str!("../../../examples/import.pd");

mod bare;
mod comparison;
mod errors;
mod examples;
mod filter;
mod length;
mod let_forms;
mod list_accessors;
mod map;
mod power;
mod programs;
mod range;

fn num(n: f64) -> Value {
    Value::Num(n, Span::default())
}

// `Value` and `EvalError` both carry a `Span` now, and both derive `PartialEq`
// over it. Real spans from parsing never equal the `Span::default()` placeholder
// tests build expected values with, so every comparison helper normalizes spans
// away before handing values back to the tests.
fn strip_value_span(v: &Value) -> Value {
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

/// Run a multi-expression program string through the full pipeline with the
/// stdlib pre-loaded, matching what the binary does.  Returns the last value.
fn run(program: &str) -> Value {
    let env = Rc::new(RefCell::new(Env::default()));
    process(STD_LIB, env.clone()).expect("stdlib failed to load");
    let mut results = process(program, env.clone()).expect("program failed to run");
    strip_value_span(&results.pop().expect("program produced no values"))
}

/// Run with stdlib + macros.pd loaded.  Returns the last value.
fn run_macros(program: &str) -> Value {
    let env = Rc::new(RefCell::new(Env::default()));
    process(STD_LIB, env.clone()).expect("stdlib failed to load");
    process(MACROS_LIB, env.clone()).expect("macros failed to load");
    let mut results = process(program, env.clone()).expect("program failed to run");
    strip_value_span(&results.pop().expect("program produced no values"))
}

/// Run without stdlib (raw builtins only).  Returns the last value.
fn run_bare(program: &str) -> Value {
    let env = Rc::new(RefCell::new(Env::default()));
    let mut results = process(program, env).expect("program failed to run");
    strip_value_span(&results.pop().expect("program produced no values"))
}

/// Run a program and expect it to return an error.
fn run_err(program: &str) -> anyhow::Error {
    let env = Rc::new(RefCell::new(Env::default()));
    process(STD_LIB, env.clone()).expect("stdlib failed to load");
    let err = process(program, env.clone()).expect_err("expected program to fail but it succeeded");
    normalize_err(err)
}

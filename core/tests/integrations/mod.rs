//! Integration tests for the paddle LISP interpreter.
//!
//! These tests exercise the full pipeline — lex → parse → lower → eval —
//! through the public `process` API, mirroring how the binary loads and
//! runs programs (stdlib first, then user code).

use std::{cell::RefCell, rc::Rc};

use paddle_core::{
    cursor::process,
    eval::{Env, value::Value},
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
    Value::Num(n)
}

/// Run a multi-expression program string through the full pipeline with the
/// stdlib pre-loaded, matching what the binary does.  Returns the last value.
fn run(program: &str) -> Value {
    let env = Rc::new(RefCell::new(Env::default()));
    process(STD_LIB, env.clone()).expect("stdlib failed to load");
    let mut results = process(program, env.clone()).expect("program failed to run");
    results.pop().expect("program produced no values")
}

/// Run with stdlib + macros.pd loaded.  Returns the last value.
fn run_macros(program: &str) -> Value {
    let env = Rc::new(RefCell::new(Env::default()));
    process(STD_LIB, env.clone()).expect("stdlib failed to load");
    process(MACROS_LIB, env.clone()).expect("macros failed to load");
    let mut results = process(program, env.clone()).expect("program failed to run");
    results.pop().expect("program produced no values")
}

/// Run without stdlib (raw builtins only).  Returns the last value.
fn run_bare(program: &str) -> Value {
    let env = Rc::new(RefCell::new(Env::default()));
    let mut results = process(program, env).expect("program failed to run");
    results.pop().expect("program produced no values")
}

/// Run a program and expect it to return an error.
fn run_err(program: &str) -> anyhow::Error {
    let env = Rc::new(RefCell::new(Env::default()));
    process(STD_LIB, env.clone()).expect("stdlib failed to load");
    process(program, env.clone()).expect_err("expected program to fail but it succeeded")
}

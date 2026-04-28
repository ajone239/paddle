//! Integration tests for the paddle LISP interpreter.
//!
//! These tests exercise the full pipeline — lex → parse → lower → eval —
//! through the public `process` API, mirroring how the binary loads and
//! runs programs (stdlib first, then user code).

use std::{cell::RefCell, rc::Rc};

use paddle::{
    cursor::process,
    eval::{Env, value::Value},
};

static STD_LIB: &str = include_str!("../examples/base.pd");
static FACT_PROGRAM: &str = include_str!("../examples/fact.pd");
static IMPORT_PROGRAM: &str = include_str!("../examples/import.pd");

fn num(n: f64) -> Value {
    Value::Num(n)
}

/// Run a multi-expression program string through the full pipeline with the
/// stdlib pre-loaded, matching what the binary does.  Returns the last value.
fn run(program: &str) -> Value {
    let env = Rc::new(RefCell::new(Env::default()));
    process(STD_LIB, env.clone()).expect("stdlib failed to load");
    let mut results = process(program, env).expect("program failed to run");
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
    process(program, env).expect_err("expected program to fail but it succeeded")
}

// ─────────────────────────────────────────────────────────────────────────────
// stdlib: comparison operators (>, <=, >=) defined in base.pd
// ─────────────────────────────────────────────────────────────────────────────

mod comparison {
    use super::*;

    #[test]
    fn greater_than_true() {
        assert_eq!(run("(> 5 3)"), Value::Bool(true));
    }

    #[test]
    fn greater_than_false() {
        assert_eq!(run("(> 3 5)"), Value::Bool(false));
    }

    #[test]
    fn greater_than_equal_is_false() {
        assert_eq!(run("(> 3 3)"), Value::Bool(false));
    }

    #[test]
    fn less_than_or_equal_strict() {
        assert_eq!(run("(<= 2 3)"), Value::Bool(true));
    }

    #[test]
    fn less_than_or_equal_equal() {
        assert_eq!(run("(<= 3 3)"), Value::Bool(true));
    }

    #[test]
    fn less_than_or_equal_false() {
        assert_eq!(run("(<= 5 3)"), Value::Bool(false));
    }

    #[test]
    fn greater_than_or_equal_strict() {
        assert_eq!(run("(>= 5 3)"), Value::Bool(true));
    }

    #[test]
    fn greater_than_or_equal_equal() {
        assert_eq!(run("(>= 3 3)"), Value::Bool(true));
    }

    #[test]
    fn greater_than_or_equal_false() {
        assert_eq!(run("(>= 2 5)"), Value::Bool(false));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// stdlib: exponentiation (^) defined in base.pd
// ─────────────────────────────────────────────────────────────────────────────

mod power {
    use super::*;

    #[test]
    fn power_zero_exponent() {
        assert_eq!(run("(^ 5 0)"), num(1.0));
    }

    #[test]
    fn power_one_exponent() {
        assert_eq!(run("(^ 7 1)"), num(7.0));
    }

    #[test]
    fn square() {
        assert_eq!(run("(^ 3 2)"), num(9.0));
    }

    #[test]
    fn large_power() {
        assert_eq!(run("(^ 2 10)"), num(1024.0));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// stdlib: length / len defined in base.pd
// ─────────────────────────────────────────────────────────────────────────────

mod length {
    use super::*;

    #[test]
    fn empty_list() {
        assert_eq!(run("(length '())"), num(0.0));
    }

    #[test]
    fn single_element() {
        assert_eq!(run("(length '(99))"), num(1.0));
    }

    #[test]
    fn three_elements() {
        assert_eq!(run("(length '(1 2 3))"), num(3.0));
    }

    #[test]
    fn len_is_alias_for_length() {
        assert_eq!(run("(len '(1 2 3))"), num(3.0));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// stdlib: cadr / caddr / cadddr defined in base.pd
// ─────────────────────────────────────────────────────────────────────────────

mod list_accessors {
    use super::*;

    #[test]
    fn cadr_is_second_element() {
        assert_eq!(run("(cadr '(10 20 30))"), num(20.0));
    }

    #[test]
    fn caddr_is_third_element() {
        assert_eq!(run("(caddr '(10 20 30))"), num(30.0));
    }

    #[test]
    fn cadddr_is_fourth_element() {
        assert_eq!(run("(cadddr '(10 20 30 40))"), num(40.0));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// stdlib: range defined in base.pd
// ─────────────────────────────────────────────────────────────────────────────

mod range {
    use super::*;

    #[test]
    fn range_zero_is_empty() {
        assert_eq!(run("(length (range 0))"), num(0.0));
    }

    #[test]
    fn range_length_equals_n() {
        assert_eq!(run("(length (range 5))"), num(5.0));
    }

    #[test]
    fn range_starts_at_zero() {
        assert_eq!(run("(car (range 5))"), num(0.0));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// stdlib: map defined in base.pd
// ─────────────────────────────────────────────────────────────────────────────

mod map {
    use super::*;

    #[test]
    fn map_on_empty_list_returns_empty() {
        assert_eq!(run("(map (lambda (x) (* x 2)) '())"), Value::List(vec![]));
    }

    #[test]
    fn map_preserves_length() {
        assert_eq!(
            run("(length (map (lambda (x) (* x 2)) '(1 2 3)))"),
            num(3.0)
        );
    }

    #[test]
    fn map_transforms_first_element() {
        assert_eq!(run("(car (map (lambda (x) (* x 2)) '(1 2 3)))"), num(2.0));
    }

    #[test]
    fn map_works_with_named_function() {
        assert_eq!(
            run("(def (square x) (* x x)) (car (map square '(3 4 5)))"),
            num(9.0)
        );
    }

    #[test]
    fn map_over_range() {
        // (map (lambda (x) (* x x)) (range 4)) has length 4
        assert_eq!(
            run("(length (map (lambda (x) (* x x)) (range 4)))"),
            num(4.0)
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// stdlib: filter defined in base.pd
// ─────────────────────────────────────────────────────────────────────────────

mod filter {
    use super::*;

    #[test]
    fn filter_on_empty_list_returns_empty() {
        assert_eq!(run("(filter (lambda (x) #t) '())"), Value::List(vec![]));
    }

    #[test]
    fn filter_remove_all() {
        assert_eq!(run("(length (filter (lambda (x) #f) '(1 2 3)))"), num(0.0));
    }

    #[test]
    fn filter_keep_all() {
        assert_eq!(run("(length (filter (lambda (x) #t) '(1 2 3)))"), num(3.0));
    }

    #[test]
    fn filter_count_passing_elements() {
        // keep x where x > 2: 3, 4, 5 → length 3
        assert_eq!(
            run("(length (filter (lambda (x) (< 2 x)) '(1 2 3 4 5)))"),
            num(3.0)
        );
    }

    #[test]
    fn filter_first_passing_element() {
        // keep x where x > 2: first passing element is 3
        assert_eq!(
            run("(car (filter (lambda (x) (< 2 x)) '(1 2 3 4 5)))"),
            num(3.0)
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// multi-expression programs — testing `process()` as a pipeline
// ─────────────────────────────────────────────────────────────────────────────

mod programs {
    use super::*;

    #[test]
    fn process_returns_a_value_per_expression() {
        let env = Rc::new(RefCell::new(Env::default()));
        let results = process("(+ 1 2) (* 3 4) (- 10 5)", env).unwrap();
        assert_eq!(results, vec![num(3.0), num(12.0), num(5.0)]);
    }

    #[test]
    fn definition_contributes_nil_to_result_list() {
        let env = Rc::new(RefCell::new(Env::default()));
        let results = process("(def x 42)", env).unwrap();
        assert_eq!(results, vec![Value::Nil]);
    }

    #[test]
    fn definitions_visible_to_later_expressions_in_same_call() {
        assert_eq!(run_bare("(def (double x) (* x 2)) (double 21)"), num(42.0));
    }

    #[test]
    fn env_persists_across_separate_process_calls() {
        let env = Rc::new(RefCell::new(Env::default()));
        process("(def (inc x) (+ x 1))", env.clone()).expect("first call failed");
        let mut r = process("(inc 41)", env).expect("second call failed");
        assert_eq!(r.pop().unwrap(), num(42.0));
    }

    #[test]
    fn chained_function_definitions() {
        assert_eq!(
            run("(def (double x) (* x 2))
                 (def (quad x) (double (double x)))
                 (quad 5)"),
            num(20.0)
        );
    }

    #[test]
    fn recursive_factorial_inline() {
        assert_eq!(
            run("(def (fact n) (if (< n 1) 1 (* n (fact (- n 1))))) (fact 7)"),
            num(5040.0)
        );
    }

    #[test]
    fn higher_order_inline() {
        assert_eq!(
            run("(def (apply-twice f x) (f (f x)))
                 (apply-twice (lambda (x) (+ x 3)) 10)"),
            num(16.0)
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// programs without stdlib (raw builtin layer only)
// ─────────────────────────────────────────────────────────────────────────────

mod bare {
    use super::*;

    #[test]
    fn arithmetic() {
        assert_eq!(run_bare("(+ (* 3 4) (- 10 4))"), num(18.0));
    }

    #[test]
    fn closure() {
        assert_eq!(
            run_bare(
                "(def (make-adder n) (lambda (x) (+ x n)))
                 (def add10 (make-adder 10))
                 (add10 32)"
            ),
            num(42.0)
        );
    }

    #[test]
    fn list_operations() {
        assert_eq!(run_bare("(car (cdr (list 10 20 30)))"), num(20.0));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// example files from examples/
// ─────────────────────────────────────────────────────────────────────────────

mod examples {
    use super::*;

    #[test]
    fn stdlib_loads_without_error() {
        let env = Rc::new(RefCell::new(Env::default()));
        process(STD_LIB, env).expect("base.pd failed to load");
    }

    #[test]
    fn fact_program_produces_ten_factorial() {
        // examples/fact.pd defines `fact` and calls `(fact 10)` → 3628800
        let env = Rc::new(RefCell::new(Env::default()));
        let mut results = process(FACT_PROGRAM, env).expect("fact.pd failed");
        assert_eq!(results.pop().unwrap(), num(3628800.0));
    }

    #[test]
    fn import_program_produces_ten_factorial() {
        // examples/fact.pd defines `fact` and calls `(fact 10)` → 3628800
        let env = Rc::new(RefCell::new(Env::default()));
        let mut results = process(IMPORT_PROGRAM, env).expect("fact.pd failed");
        assert_eq!(results.pop().unwrap(), num(3628800.0));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// error propagation through the full pipeline
// ─────────────────────────────────────────────────────────────────────────────

mod errors {
    use super::*;
    use paddle::eval::EvalError;

    #[test]
    fn undefined_symbol_bubbles_up() {
        let err = run_err("totally-undefined-symbol");
        assert_eq!(
            err.downcast_ref::<EvalError>(),
            Some(&EvalError::SymbolUndefined(
                "totally-undefined-symbol".into()
            ))
        );
    }

    #[test]
    fn wrong_arg_count_bubbles_up() {
        let err = run_err("(def (f x) (+ x 1)) (f 1 2)");
        assert_eq!(
            err.downcast_ref::<EvalError>(),
            Some(&EvalError::BadFunctionArgCount(1, 2))
        );
    }

    #[test]
    fn bad_lambda_missing_body_bubbles_up() {
        let err = run_err("(lambda (x))");
        assert_eq!(
            err.downcast_ref::<EvalError>(),
            Some(&EvalError::BadLambdaArgs)
        );
    }

    #[test]
    fn error_in_later_expression_stops_program() {
        // First expression succeeds; second expression should fail
        let env = Rc::new(RefCell::new(Env::default()));
        process(STD_LIB, env.clone()).expect("stdlib failed");
        let result = process("(+ 1 2) undefined-sym", env);
        assert!(result.is_err());
    }
}

use super::*;

#[test]
fn define_func_and_call() {
    assert_eq!(
        eval_str_env(&["(def (double x) (* x 2))", "(double 3)"]),
        num(6.0)
    );
}

#[test]
fn define_func_scope() {
    let err = eval_env_err(&["(def (double x) (* x 2))", "(double 3)", "x"]);
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::SymbolUndefined("x".into()))
    );
}

#[test]
fn define_func_shadow() {
    assert_eq!(
        eval_str_env(&["(def x 10)", "(def (double x) (* x 2))", "(double 3)", "x"]),
        num(10.0)
    );
}

#[test]
fn define_func_two_args() {
    assert_eq!(
        eval_str_env(&["(def (add x y) (+ x y))", "(add 3 4)"]),
        num(7.0)
    );
}

#[test]
fn define_func_no_args() {
    assert_eq!(
        eval_str_env(&["(def (forty-two) 42)", "(forty-two)"]),
        num(42.0)
    );
}

#[test]
fn define_func_multi_body() {
    assert_eq!(
        eval_str_env(&["(def (f x) (+ x 1) (* x 2))", "(f 3)"]),
        num(6.0)
    );
}

#[test]
fn define_func_returns_nil() {
    assert_eq!(eval_str("(def (f x) (+ x 1))"), Value::Nil);
}

#[test]
fn define_func_fact() {
    assert_eq!(
        eval_str_env(&vec![
            "(def (fact n) (if (< n 1) 1 (* n (fact (- n 1)))))",
            "(fact 5)"
        ]),
        Value::Num(120.0)
    );
}

#[test]
fn define_func_fact_cute() {
    assert_eq!(
        eval_str_env(&vec![
            "
(def (fact n)
    (if (< n 1)
     1
     (* n (fact (- n 1)))))
",
            "(fact 5)"
        ]),
        Value::Num(120.0)
    );
}

#[test]
fn define_func_nested_call() {
    assert_eq!(
        eval_str_env(&[
            "(def (double x) (* x 2))",
            "(def (quad x) (double (double x)))",
            "(quad 3)"
        ]),
        num(12.0)
    );
}

#[test]
fn define_func_wrong_arity() {
    let err = eval_env_err(&["(def (f x) (+ x 1))", "(f 1 2)"]);
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadFunctionArgCount(1, 2))
    );
}

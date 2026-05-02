use super::*;

#[test]
fn define_and_resolve() {
    assert_eq!(eval_str_env(&vec!["(def x 5)", "(+ x 1)"]), num(6.0));
}

#[test]
fn redefine() {
    assert_eq!(eval_str_env(&vec!["(def x 1)", "(def x 2)", "x"]), num(2.0));
}

#[test]
fn define_returns_nil() {
    assert_eq!(eval_str("(def x 5)"), Value::Nil);
}

#[test]
fn define_expression_value() {
    assert_eq!(eval_str_env(&vec!["(def x (+ 1 2))", "x"]), num(3.0));
}

#[test]
fn undefined_symbol() {
    let err = eval_err("x");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::SymbolUndefined("x".into()))
    );
}

use super::*;

#[test]
fn set_bang_basic() {
    assert_eq!(eval_str_env(&["(def x 3)", "(set! x 5)", "x"]), num(5.0))
}

#[test]
fn set_bang_from_closure() {
    assert_eq!(
        eval_str_env(&["(def x 3)", "((.\\ () (set! x 5)))", "x"]),
        num(5.0)
    )
}

#[test]
fn set_bang_from_func() {
    assert_eq!(
        eval_str_env(&["(def x 3)", "(def (t) (set! x (+ x 1)))", "(t)", "(t)", "x"]),
        num(5.0)
    )
}

#[test]
fn set_bang_returns_no_print() {
    assert_eq!(eval_str_env(&["(def x 3)", "(set! x 5)"]), Value::NoPrint)
}

#[test]
fn set_bang_undefined_variable() {
    let err = eval_env_err(&["(set! y 5)"]);
    assert!(err.to_string().contains("isn't in scope"));
}

#[test]
fn set_bang_too_few_args() {
    let err = eval_env_err(&["(set! x)"]);
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadSetBangArgs)
    );
}

#[test]
fn set_bang_too_many_args() {
    let err = eval_env_err(&["(def x 3)", "(set! x 5 6)"]);
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadSetBangArgs)
    );
}

#[test]
fn set_bang_non_symbol_head() {
    let err = eval_env_err(&["(set! 5 3)"]);
    assert!(err.to_string().contains("Bad set! head"));
}

#[test]
fn set_bang_builtin_not_in_scope() {
    let err = eval_env_err(&["(set! + 1)"]);
    assert!(err.to_string().contains("isn't in scope"));
}

#[test]
fn set_bang_through_nested_closures() {
    assert_eq!(
        eval_str_env(&["(def x 3)", "(((.\\ () (.\\ () (set! x 5)))))", "x"]),
        num(5.0)
    )
}

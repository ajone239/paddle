use super::*;

#[test]
fn if_true_branch() {
    assert_eq!(eval_str("(if #t 1 2)"), num(1.0));
}

#[test]
fn if_false_branch() {
    assert_eq!(eval_str("(if #f 1 2)"), num(2.0));
}

#[test]
fn if_truthy_num() {
    assert_eq!(eval_str("(if 1 10 20)"), num(10.0));
}

#[test]
fn if_falsy_zero() {
    assert_eq!(eval_str("(if 0 10 20)"), num(20.0));
}

#[test]
fn if_falsy_nil() {
    assert_eq!(eval_str("(if nil 10 20)"), num(20.0));
}

#[test]
fn if_condition_is_expression() {
    assert_eq!(eval_str("(if (< 1 2) 10 20)"), num(10.0));
}

#[test]
fn if_only_evaluates_true_branch() {
    eval_str_env(&["(def x 1)", "(if #t x undefined)"]);
}

#[test]
fn if_only_evaluates_false_branch() {
    eval_str_env(&["(def x 1)", "(if #f undefined x)"]);
}

#[test]
fn if_nested() {
    assert_eq!(eval_str("(if #t (if #f 1 2) 3)"), num(2.0));
}

#[test]
fn if_no_else_true() {
    let err = eval_err("(if #t 42)");
    assert_eq!(err.downcast_ref::<EvalError>(), Some(&EvalError::BadIfArgs));
}

#[test]
fn if_no_else_false() {
    let err = eval_err("(if #f 42)");
    assert_eq!(err.downcast_ref::<EvalError>(), Some(&EvalError::BadIfArgs));
}

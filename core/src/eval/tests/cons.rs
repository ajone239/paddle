use super::*;
use crate::eval::env::BuiltinError;

#[test]
fn cons_two_atoms() {
    assert_eq!(eval_str("(cons 1 2)"), cons(num(1.0), num(2.0)));
}

#[test]
fn cons_with_nil_tail() {
    assert_eq!(eval_str("(cons 1 nil)"), cons(num(1.0), Value::Nil));
}

#[test]
fn cons_with_list_tail() {
    // cons does not flatten — tail stays as a nested list
    assert_eq!(
        eval_str("(cons 1 '(2 3))"),
        cons(num(1.0), cons(num(2.0), cons(num(3.0), Value::Nil)))
    );
}

#[test]
fn cons_wrong_arity_one() {
    let err = eval_err("(cons 1)");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::WrongConsArgCount)
    );
}

#[test]
fn cons_wrong_arity_three() {
    let err = eval_err("(cons 1 2 3)");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::WrongConsArgCount)
    );
}

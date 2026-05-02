use super::*;
use crate::eval::env::BuiltinError;

#[test]
fn car_of_cons() {
    assert_eq!(eval_str("(car (cons 1 2))"), num(1.0));
}

#[test]
fn car_of_quoted_list() {
    assert_eq!(eval_str("(car '(10 20 30))"), num(10.0));
}

#[test]
fn car_of_single_element_list() {
    assert_eq!(eval_str("(car '(42))"), num(42.0));
}

#[test]
fn car_of_nil() {
    let err = eval_err("(car nil)");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::WrongCarArgType)
    );
}

#[test]
fn car_of_atom() {
    let err = eval_err("(car 1)");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::WrongCarArgType)
    );
}

#[test]
fn car_wrong_arity() {
    let err = eval_err("(car '(1) '(2))");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::WrongCarArgCount(2))
    );
}

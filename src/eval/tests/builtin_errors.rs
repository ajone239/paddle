use super::*;
use crate::eval::env::BuiltinError;

#[test]
fn car_empty_list() {
    // (car nil) hits WrongCarArgType; need a real empty list
    let err = eval_err("(car '())");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::CarOnEmptyList)
    );
}

#[test]
fn car_arity() {
    // (car nil) hits WrongCarArgType; need a real empty list
    let err = eval_err("(car '() '())");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::WrongCarArgCount)
    );
}

#[test]
fn cdr_empty_list() {
    let err = eval_err("(cdr '())");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::CdrOnEmptyList)
    );
}

#[test]
fn cdr_arity() {
    let err = eval_err("(cdr '() '())");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::WrongCdrArgCount)
    );
}

#[test]
fn expected_num_arg() {
    let err = eval_err(r#"(+ "foo" 1)"#);
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::ExpectedNumArg)
    );
}

#[test]
fn minus_no_args() {
    let err = eval_err("(-)");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::NoInitforMinus)
    );
}

#[test]
fn div_no_args() {
    let err = eval_err("(/)");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::NoInitforDiv)
    );
}

#[test]
fn lt_bad_arg_types() {
    let err = eval_err(r#"(< "a" "b")"#);
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::ExpectedNumArg)
    );
}

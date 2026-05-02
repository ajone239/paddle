use super::*;
use crate::eval::env::BuiltinError;

#[test]
fn cdr_of_cons() {
    // cdr of (cons 1 2) is List([2]), not the atom 2
    assert_eq!(eval_str("(cdr (cons 1 2))"), Value::List(vec![num(2.0)]));
}

#[test]
fn cdr_of_quoted_list() {
    assert_eq!(
        eval_str("(cdr '(1 2 3))"),
        Value::List(vec![num(2.0), num(3.0)])
    );
}

#[test]
fn cdr_of_single_element_list_is_empty_list() {
    // cdr drops the head, leaving an empty Vec — not Nil
    assert_eq!(eval_str("(cdr '(1))"), Value::List(vec![]));
}

#[test]
fn cdr_of_nil() {
    let err = eval_err("(cdr nil)");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::WrongCdrArgType)
    );
}

#[test]
fn cdr_of_atom() {
    let err = eval_err("(cdr 1)");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::WrongCdrArgType)
    );
}

#[test]
fn cdr_wrong_arity() {
    let err = eval_err("(cdr '(1) '(2))");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::WrongCdrArgCount(2))
    );
}

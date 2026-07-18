use super::*;
use crate::eval::env::BuiltinError;

#[test]
fn cdr_of_cons() {
    assert_eq!(eval_str("(cdr (cons 1 2))"), num(2.0));
}

#[test]
fn cdr_of_list() {
    assert_eq!(
        eval_str("(cdr '(1 2))"),
        cons(num(2.0), Value::Nil(Span::default()))
    );
}

#[test]
fn cdr_of_quoted_list() {
    assert_eq!(
        eval_str("(cdr '(1 2 3))"),
        cons(num(2.0), cons(num(3.0), Value::Nil(Span::default())))
    );
}

#[test]
fn cdr_of_single_element_list_is_empty_list() {
    assert_eq!(eval_str("(cdr '(1))"), Value::Nil(Span::default()))
}

#[test]
fn cdr_of_nil() {
    let err = eval_err("(cdr nil)");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::CdrOnEmptyList)
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
fn cdr_of_list_with_nil_head() {
    assert_eq!(
        eval_str("(cdr '(nil 1))"),
        cons(num(1.0), Value::Nil(Span::default()))
    );
}

#[test]
fn cdr_wrong_arity() {
    let err = eval_err("(cdr '(1) '(2))");
    assert_eq!(
        err.downcast_ref::<BuiltinError>(),
        Some(&BuiltinError::WrongCdrArgCount)
    );
}

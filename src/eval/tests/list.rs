use super::*;

#[test]
fn list_empty() {
    assert_eq!(eval_str("(list)"), Value::List(vec![]));
}

#[test]
fn list_single() {
    assert_eq!(eval_str("(list 1)"), Value::List(vec![num(1.0)]));
}

#[test]
fn list_multiple() {
    assert_eq!(
        eval_str("(list 1 2 3)"),
        Value::List(vec![num(1.0), num(2.0), num(3.0)])
    );
}

#[test]
fn list_evaluates_args() {
    assert_eq!(
        eval_str("(list (+ 1 1) (* 2 3))"),
        Value::List(vec![num(2.0), num(6.0)])
    );
}

#[test]
fn list_mixed_types() {
    assert_eq!(
        eval_str("(list 1 #t nil)"),
        Value::List(vec![num(1.0), Value::Bool(true), Value::Nil])
    );
}

#[test]
fn car_of_list() {
    assert_eq!(eval_str("(car (list 10 20 30))"), num(10.0));
}

#[test]
fn cdr_of_list() {
    assert_eq!(
        eval_str("(cdr (list 1 2 3))"),
        Value::List(vec![num(2.0), num(3.0)])
    );
}

#[test]
fn list_with_quoted_symbol() {
    assert_eq!(
        eval_str("(list 'a 'b)"),
        Value::List(vec![
            Value::Symbol("a".to_owned()),
            Value::Symbol("b".to_owned())
        ])
    );
}

#[test]
fn list_with_quoted_list_arg() {
    assert_eq!(
        eval_str("(list '(1 2) 3)"),
        Value::List(vec![Value::List(vec![num(1.0), num(2.0)]), num(3.0)])
    );
}

#[test]
fn list_of_lists() {
    assert_eq!(
        eval_str("(list (list 1 2) (list 3 4))"),
        Value::List(vec![
            Value::List(vec![num(1.0), num(2.0)]),
            Value::List(vec![num(3.0), num(4.0)]),
        ])
    );
}

#[test]
fn quote_of_list_call_suppresses_eval() {
    assert_eq!(
        eval_str("'(list 1 2)"),
        Value::List(vec![Value::Symbol("list".to_owned()), num(1.0), num(2.0)])
    );
}

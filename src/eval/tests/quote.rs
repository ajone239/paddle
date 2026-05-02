use super::*;

#[test]
fn quote_symbol() {
    assert_eq!(eval_str("(quote x)"), Value::Symbol("x".to_owned()));
}

#[test]
fn quote_number() {
    assert_eq!(eval_str("(quote 42)"), num(42.0));
    assert_eq!(eval_str("'42"), num(42.0));
}

#[test]
fn quote_nil() {
    assert_eq!(eval_str("(quote nil)"), Value::Nil);
}

#[test]
fn quote_list() {
    assert_eq!(
        eval_str("(quote (1 2 3))"),
        Value::List(vec![num(1.0), num(2.0), num(3.0)])
    );
}

#[test]
fn quote_suppresses_eval() {
    assert_eq!(
        eval_str("(quote (+ 1 2))"),
        Value::List(vec![Value::Symbol("+".to_owned()), num(1.0), num(2.0),])
    );
}

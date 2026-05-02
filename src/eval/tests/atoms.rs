use super::*;

#[test]
fn integer_literal() {
    assert_eq!(eval_str("42"), num(42.0));
}

#[test]
fn float_literal() {
    assert_eq!(eval_str("3.14"), num(3.14));
}

#[test]
fn negative_literal() {
    assert_eq!(eval_str("-7"), num(-7.0));
}

#[test]
fn nil_literal() {
    assert_eq!(eval_str("nil"), Value::Nil);
}

#[test]
fn true_literal() {
    assert_eq!(eval_str("#t"), Value::Bool(true));
}

#[test]
fn false_literal() {
    assert_eq!(eval_str("#f"), Value::Bool(false));
}

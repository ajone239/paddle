use super::*;

#[test]
fn add_two() {
    assert_eq!(eval_str("(+ 1 2)"), num(3.0));
}

#[test]
fn add_three() {
    assert_eq!(eval_str("(+ 1 2 3)"), num(6.0));
}

#[test]
fn add_no_args() {
    assert_eq!(eval_str("(+)"), num(0.0));
}

#[test]
fn add_one_arg() {
    assert_eq!(eval_str("(+ 5)"), num(5.0));
}

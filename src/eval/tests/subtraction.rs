use super::*;

#[test]
fn sub_two() {
    assert_eq!(eval_str("(- 10 3)"), num(7.0));
}

#[test]
fn sub_three() {
    assert_eq!(eval_str("(- 10 3 2)"), num(5.0));
}

#[test]
fn sub_one_arg() {
    assert_eq!(eval_str("(- 5)"), num(5.0));
}

use super::*;

#[test]
fn mul_two() {
    assert_eq!(eval_str("(* 3 4)"), num(12.0));
}

#[test]
fn mul_no_args() {
    assert_eq!(eval_str("(*)"), num(1.0));
}

#[test]
fn mul_one_arg() {
    assert_eq!(eval_str("(* 7)"), num(7.0));
}

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

#[test]
fn div_two() {
    assert_eq!(eval_str("(/ 10 2)"), num(5.0));
}

#[test]
fn div_three() {
    assert_eq!(eval_str("(/ 24 4 3)"), num(2.0));
}

#[test]
fn nested_add() {
    assert_eq!(eval_str("(+ 1 (+ 2 3))"), num(6.0));
}

#[test]
fn nested_mixed() {
    assert_eq!(eval_str("(* (+ 1 2) (- 5 2))"), num(9.0));
}

#[test]
fn deeply_nested() {
    assert_eq!(eval_str("(+ 1 (* 2 (- 10 (/ 6 2))))"), num(15.0));
}

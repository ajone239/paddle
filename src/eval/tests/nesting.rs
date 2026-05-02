use super::*;

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

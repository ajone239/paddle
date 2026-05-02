use super::*;

#[test]
fn div_two() {
    assert_eq!(eval_str("(/ 10 2)"), num(5.0));
}

#[test]
fn div_three() {
    assert_eq!(eval_str("(/ 24 4 3)"), num(2.0));
}

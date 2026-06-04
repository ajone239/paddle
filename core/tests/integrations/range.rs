use super::*;

#[test]
fn range_zero_is_empty() {
    assert_eq!(run("(length (range 0))"), num(0.0));
}

#[test]
fn range_length_equals_n() {
    assert_eq!(run("(length (range 5))"), num(5.0));
}

#[test]
fn range_starts_at_zero() {
    assert_eq!(run("(car (range 5))"), num(0.0));
}

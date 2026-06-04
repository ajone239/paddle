use super::*;

#[test]
fn power_zero_exponent() {
    assert_eq!(run("(^ 5 0)"), num(1.0));
}

#[test]
fn power_one_exponent() {
    assert_eq!(run("(^ 7 1)"), num(7.0));
}

#[test]
fn square() {
    assert_eq!(run("(^ 3 2)"), num(9.0));
}

#[test]
fn large_power() {
    assert_eq!(run("(^ 2 10)"), num(1024.0));
}

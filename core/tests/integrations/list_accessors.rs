use super::*;

#[test]
fn cadr_is_second_element() {
    assert_eq!(run("(cadr '(10 20 30))"), num(20.0));
}

#[test]
fn caddr_is_third_element() {
    assert_eq!(run("(caddr '(10 20 30))"), num(30.0));
}

#[test]
fn cadddr_is_fourth_element() {
    assert_eq!(run("(cadddr '(10 20 30 40))"), num(40.0));
}

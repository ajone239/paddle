use super::*;

#[test]
fn empty_list() {
    assert_eq!(run("(length '())"), num(0.0));
}

#[test]
fn single_element() {
    assert_eq!(run("(length '(99))"), num(1.0));
}

#[test]
fn three_elements() {
    assert_eq!(run("(length '(1 2 3))"), num(3.0));
}

#[test]
fn len_is_alias_for_length() {
    assert_eq!(run("(len '(1 2 3))"), num(3.0));
}

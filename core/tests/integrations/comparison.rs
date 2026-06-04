use super::*;

#[test]
fn greater_than_true() {
    assert_eq!(run("(> 5 3)"), Value::Bool(true));
}

#[test]
fn greater_than_false() {
    assert_eq!(run("(> 3 5)"), Value::Bool(false));
}

#[test]
fn greater_than_equal_is_false() {
    assert_eq!(run("(> 3 3)"), Value::Bool(false));
}

#[test]
fn less_than_or_equal_strict() {
    assert_eq!(run("(<= 2 3)"), Value::Bool(true));
}

#[test]
fn less_than_or_equal_equal() {
    assert_eq!(run("(<= 3 3)"), Value::Bool(true));
}

#[test]
fn less_than_or_equal_false() {
    assert_eq!(run("(<= 5 3)"), Value::Bool(false));
}

#[test]
fn greater_than_or_equal_strict() {
    assert_eq!(run("(>= 5 3)"), Value::Bool(true));
}

#[test]
fn greater_than_or_equal_equal() {
    assert_eq!(run("(>= 3 3)"), Value::Bool(true));
}

#[test]
fn greater_than_or_equal_false() {
    assert_eq!(run("(>= 2 5)"), Value::Bool(false));
}

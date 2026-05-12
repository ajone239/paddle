use super::*;

#[test]
fn empty_list() {
    assert_eq!(eval_str("()"), Value::Nil);
}

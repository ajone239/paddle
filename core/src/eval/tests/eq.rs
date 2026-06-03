use super::*;

#[test]
fn eq_numbers_equal() {
    assert_eq!(eval_str("(= 1 1)"), Value::Bool(true));
}

#[test]
fn eq_numbers_unequal() {
    assert_eq!(eval_str("(= 1 2)"), Value::Bool(false));
}

#[test]
fn eq_strings_equal() {
    assert_eq!(eval_str(r#"(= "hi" "hi")"#), Value::Bool(true));
}

#[test]
fn eq_strings_unequal() {
    assert_eq!(eval_str(r#"(= "hi" "bye")"#), Value::Bool(false));
}

#[test]
fn eq_symbols_equal() {
    assert_eq!(eval_str("(= 'foo 'foo)"), Value::Bool(true));
}

#[test]
fn eq_symbols_unequal() {
    assert_eq!(eval_str("(= 'foo 'bar)"), Value::Bool(false));
}

#[test]
fn eq_symbol_and_string_same_content() {
    assert_eq!(eval_str(r#"(= 'foo "foo")"#), Value::Bool(true));
}

#[test]
fn eq_string_and_symbol_same_content() {
    assert_eq!(eval_str(r#"(= "foo" 'foo)"#), Value::Bool(true));
}

#[test]
fn eq_symbol_and_string_different_content() {
    assert_eq!(eval_str(r#"(= 'foo "bar")"#), Value::Bool(false));
}

#[test]
fn eq_nil_nil() {
    assert_eq!(eval_str("(= nil nil)"), Value::Bool(true));
}

#[test]
fn eq_nil_empty_list() {
    // '() and nil are the same value
    assert_eq!(eval_str("(= nil '())"), Value::Bool(true));
}

#[test]
fn eq_nil_number() {
    assert_eq!(eval_str("(= nil 1)"), Value::Bool(false));
}

#[test]
fn eq_number_nil() {
    assert_eq!(eval_str("(= 1 nil)"), Value::Bool(false));
}

#[test]
fn eq_nil_false() {
    assert_eq!(eval_str("(= nil #f)"), Value::Bool(false));
}

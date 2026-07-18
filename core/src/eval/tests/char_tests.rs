use super::*;

// char from number

#[test]
fn char_from_num() {
    assert_eq!(eval_str("(char 65)"), Value::Char(65, Span::default()));
}

#[test]
fn char_from_num_displays_as_letter() {
    assert_eq!(eval_str("(char 65)").to_string(), "'A'");
}

#[test]
fn char_from_max_byte() {
    assert_eq!(eval_str("(char 255)"), Value::Char(255, Span::default()));
}

// char from string / symbol

#[test]
fn char_from_string() {
    assert_eq!(eval_str(r#"(char "A")"#), Value::Char(65, Span::default()));
}

#[test]
fn char_from_symbol() {
    assert_eq!(eval_str("(char 'A)"), Value::Char(65, Span::default()));
}

// errors: wrong shape of str/sym

#[test]
fn char_from_multi_char_string_errors() {
    eval_err(r#"(char "AB")"#);
}

#[test]
fn char_from_multi_char_symbol_errors() {
    eval_err("(char 'AB)");
}

// errors: wrong type

#[test]
fn char_from_nil_errors() {
    eval_err("(char nil)");
}

#[test]
fn char_from_bool_errors() {
    eval_err("(char #t)");
}

#[test]
fn char_from_list_errors() {
    eval_err("(char '(1))");
}

// errors: out of range numbers

#[test]
fn char_from_zero_errors() {
    eval_err("(char -1)");
}

#[test]
fn char_from_too_large_num_errors() {
    eval_err("(char 256)");
}

// errors: arity

#[test]
fn char_no_args_errors() {
    eval_err("(char)");
}

#[test]
fn char_too_many_args_errors() {
    eval_err("(char 65 66)");
}

// truthiness

#[test]
fn char_is_truthy() {
    assert_eq!(
        eval_str(r#"(if (char 65) "yes" "no")"#),
        Value::Str("yes".into(), Span::default())
    );
}

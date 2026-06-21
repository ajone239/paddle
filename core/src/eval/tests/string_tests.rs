use super::*;

// string-length

#[test]
fn string_length_of_string() {
    assert_eq!(eval_str(r#"(string-length "hello")"#), num(5.0));
}

#[test]
fn string_length_of_empty_string() {
    assert_eq!(eval_str(r#"(string-length "")"#), num(0.0));
}

#[test]
fn string_length_of_symbol() {
    assert_eq!(eval_str("(string-length 'hello)"), num(5.0));
}

#[test]
fn string_length_no_args_errors() {
    eval_err("(string-length)");
}

#[test]
fn string_length_too_many_args_errors() {
    eval_err(r#"(string-length "a" "b")"#);
}

#[test]
fn string_length_of_num_errors() {
    eval_err("(string-length 5)");
}

// string-ref

#[test]
fn string_ref_first_char() {
    assert_eq!(eval_str(r#"(string-ref "hello" 0)"#), Value::Char(b'h'));
}

#[test]
fn string_ref_last_char() {
    assert_eq!(eval_str(r#"(string-ref "hello" 4)"#), Value::Char(b'o'));
}

#[test]
fn string_ref_out_of_range_errors() {
    eval_err(r#"(string-ref "hello" 10)"#);
}

#[test]
fn string_ref_non_num_idx_errors() {
    eval_err(r#"(string-ref "hello" "0")"#);
}

#[test]
fn string_ref_too_few_args_errors() {
    eval_err(r#"(string-ref "hello")"#);
}

// substring

#[test]
fn substring_with_from_and_to() {
    assert_eq!(
        eval_str(r#"(substring "hello" 1 3)"#),
        Value::Str("el".into())
    );
}

#[test]
fn substring_without_to_goes_to_end() {
    assert_eq!(
        eval_str(r#"(substring "hello" 1)"#),
        Value::Str("ello".into())
    );
}

#[test]
fn substring_to_beyond_len_is_clamped() {
    assert_eq!(
        eval_str(r#"(substring "hello" 1 100)"#),
        Value::Str("ello".into())
    );
}

#[test]
fn substring_from_at_end_errors() {
    eval_err(r#"(substring "hello" 5)"#);
}

#[test]
fn substring_from_after_to_errors() {
    eval_err(r#"(substring "hello" 3 1)"#);
}

// string-append

#[test]
fn string_append_two_strings() {
    assert_eq!(
        eval_str(r#"(string-append "foo" "bar")"#),
        Value::Str("foobar".into())
    );
}

#[test]
fn string_append_strings_and_symbols() {
    assert_eq!(
        eval_str(r#"(string-append "foo" 'bar)"#),
        Value::Str("foobar".into())
    );
}

#[test]
fn string_append_no_args_is_empty_string() {
    assert_eq!(eval_str("(string-append)"), Value::Str("".into()));
}

#[test]
fn string_append_of_num_errors() {
    eval_err(r#"(string-append "foo" 5)"#);
}

// string->list

#[test]
fn string_list_basic() {
    assert_eq!(
        eval_str(r#"(string->list "ab")"#),
        Value::to_cons_list(vec![Value::Char(b'a'), Value::Char(b'b')])
    );
}

#[test]
fn string_list_of_empty_string() {
    assert_eq!(
        eval_str(r#"(string->list "")"#),
        Value::to_cons_list(vec![])
    );
}

#[test]
fn string_list_too_many_args_errors() {
    eval_err(r#"(string->list "a" "b")"#);
}

#[test]
fn string_list_of_num_errors() {
    eval_err("(string->list 5)");
}

// string->num

#[test]
fn string_num_integer() {
    assert_eq!(eval_str(r#"(string->num "42")"#), num(42.0));
}

#[test]
fn string_num_decimal() {
    assert_eq!(eval_str(r#"(string->num "3.14")"#), num(3.14));
}

#[test]
fn string_num_invalid_string_errors() {
    eval_err(r#"(string->num "abc")"#);
}

#[test]
fn string_num_too_many_args_errors() {
    eval_err(r#"(string->num "1" "2")"#);
}

// list->string

#[test]
fn list_string_of_chars() {
    assert_eq!(
        eval_str("(list->string (list (char 97) (char 98)))"),
        Value::Str("ab".into())
    );
}

#[test]
fn list_string_round_trips_with_string_list() {
    assert_eq!(
        eval_str(r#"(list->string (string->list "hello"))"#),
        Value::Str("hello".into())
    );
}

#[test]
fn list_string_of_nums_stringifies_each() {
    assert_eq!(
        eval_str("(list->string (list 1 2))"),
        Value::Str("12".into())
    );
}

#[test]
fn list_string_too_many_args_errors() {
    eval_err(r#"(list->string (list 1 2) "extra")"#);
}

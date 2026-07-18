use super::*;
use crate::eval::EvalError;

// --- position within the surrounding list ---

#[test]
fn only_element() {
    assert_eq!(
        eval_str_env(&["(def xs '(2 3))", "`(@xs)"]),
        Value::to_cons_list(vec![num(2.0), num(3.0)])
    );
}

#[test]
fn at_start() {
    assert_eq!(
        eval_str_env(&["(def xs '(1 2))", "`(@xs 3)"]),
        Value::to_cons_list(vec![num(1.0), num(2.0), num(3.0)])
    );
}

#[test]
fn at_end() {
    assert_eq!(
        eval_str_env(&["(def xs '(2 3))", "`(1 @xs)"]),
        Value::to_cons_list(vec![num(1.0), num(2.0), num(3.0)])
    );
}

#[test]
fn in_middle() {
    assert_eq!(
        eval_str_env(&["(def xs '(2 3))", "`(1 @xs 4)"]),
        Value::to_cons_list(vec![num(1.0), num(2.0), num(3.0), num(4.0)])
    );
}

// --- spliced list size ---

#[test]
fn empty_list_is_identity() {
    assert_eq!(
        eval_str_env(&["(def xs '())", "`(1 @xs 2)"]),
        Value::to_cons_list(vec![num(1.0), num(2.0)])
    );
}

#[test]
fn singleton() {
    assert_eq!(
        eval_str_env(&["(def xs '(42))", "`(@xs)"]),
        Value::to_cons_list(vec![num(42.0)])
    );
}

// --- interaction with unquote ---

#[test]
fn unquote_before_splice() {
    assert_eq!(
        eval_str_env(&["(def a 1)", "(def xs '(2 3))", "`(,a @xs)"]),
        Value::to_cons_list(vec![num(1.0), num(2.0), num(3.0)])
    );
}

#[test]
fn splice_before_unquote() {
    assert_eq!(
        eval_str_env(&["(def xs '(1 2))", "(def b 3)", "`(@xs ,b)"]),
        Value::to_cons_list(vec![num(1.0), num(2.0), num(3.0)])
    );
}

#[test]
fn splice_sandwiched_by_unquotes() {
    assert_eq!(
        eval_str_env(&["(def a 1)", "(def xs '(2 3))", "(def b 4)", "`(,a @xs ,b)"]),
        Value::to_cons_list(vec![num(1.0), num(2.0), num(3.0), num(4.0)])
    );
}

// --- multiple splices ---

#[test]
fn two_adjacent_splices() {
    assert_eq!(
        eval_str_env(&["(def xs '(1 2))", "(def ys '(3 4))", "`(@xs @ys)"]),
        Value::to_cons_list(vec![num(1.0), num(2.0), num(3.0), num(4.0)])
    );
}

#[test]
fn two_splices_separated_by_literal() {
    assert_eq!(
        eval_str_env(&["(def xs '(1 2))", "(def ys '(4 5))", "`(@xs 3 @ys)"]),
        Value::to_cons_list(vec![num(1.0), num(2.0), num(3.0), num(4.0), num(5.0)])
    );
}

// --- inline (computed) expression ---

#[test]
fn inline_expression() {
    assert_eq!(
        eval_str_env(&["`(@(list 1 2) 3)"]),
        Value::to_cons_list(vec![num(1.0), num(2.0), num(3.0)])
    );
}

// --- macro body usage ---

#[test]
fn macro_body_splice() {
    assert_eq!(
        eval_str_env(&[
            "(defm (my-progn xs...) `(progn @xs...))",
            "(my-progn (+ 1 1) (+ 2 2))"
        ]),
        num(4.0)
    );
}

// --- errors ---

#[test]
fn outside_quasiquote_errors() {
    let err = eval_env_err(&["@xs"]);
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::UnquoteOutsideQuasi(Span::default()))
    );
}

#[test]
#[should_panic(expected = "has to be a list")]
fn splice_non_list_panics() {
    eval_str_env(&["(def x 42)", "`(@x)"]);
}

use super::*;

#[test]
fn list_empty() {
    assert_eq!(eval_str("(list)"), Value::to_cons_list(vec![]));
}

#[test]
fn list_single() {
    assert_eq!(eval_str("(list 1)"), Value::to_cons_list(vec![num(1.0)]));
}

#[test]
fn list_multiple() {
    assert_eq!(
        eval_str("(list 1 2 3)"),
        Value::to_cons_list(vec![num(1.0), num(2.0), num(3.0)])
    );
}

#[test]
fn list_evaluates_args() {
    assert_eq!(
        eval_str("(list (+ 1 1) (* 2 3))"),
        Value::to_cons_list(vec![num(2.0), num(6.0)])
    );
}

#[test]
fn list_mixed_types() {
    assert_eq!(
        eval_str("(list 1 #t nil)"),
        Value::to_cons_list(vec![
            num(1.0),
            Value::Bool(true, Span::default()),
            Value::Nil(Span::default())
        ])
    );
}

#[test]
fn car_of_list() {
    assert_eq!(eval_str("(car (list 10 20 30))"), num(10.0));
}

#[test]
fn cdr_of_list() {
    assert_eq!(
        eval_str("(cdr (list 1 2 3))"),
        Value::to_cons_list(vec![num(2.0), num(3.0)])
    );
}

#[test]
fn list_with_quoted_symbol() {
    assert_eq!(
        eval_str("(list 'a 'b)"),
        Value::to_cons_list(vec![
            Value::Symbol("a".into(), Span::default()),
            Value::Symbol("b".into(), Span::default())
        ])
    );
}

#[test]
fn list_with_quoted_list_arg() {
    assert_eq!(
        eval_str("(list '(1 2) 3)"),
        Value::to_cons_list(vec![
            Value::to_cons_list(vec![num(1.0), num(2.0)]),
            num(3.0)
        ])
    );
}

#[test]
fn list_of_lists() {
    assert_eq!(
        eval_str("(list (list 1 2) (list 3 4))"),
        Value::to_cons_list(vec![
            Value::to_cons_list(vec![num(1.0), num(2.0)]),
            Value::to_cons_list(vec![num(3.0), num(4.0)]),
        ])
    );
}

#[test]
fn quote_of_list_call_suppresses_eval() {
    assert_eq!(
        eval_str("'(list 1 2)"),
        Value::to_cons_list(vec![
            Value::Symbol("list".into(), Span::default()),
            num(1.0),
            num(2.0)
        ])
    );
}

// append

#[test]
fn append_two_lists() {
    assert_eq!(
        eval_str("(append '(1 2) '(3 4))"),
        Value::to_cons_list(vec![num(1.0), num(2.0), num(3.0), num(4.0)])
    );
}

#[test]
fn append_first_empty() {
    assert_eq!(
        eval_str("(append nil '(1 2))"),
        Value::to_cons_list(vec![num(1.0), num(2.0)])
    );
}

#[test]
fn append_second_empty() {
    assert_eq!(
        eval_str("(append '(1 2) nil)"),
        Value::to_cons_list(vec![num(1.0), num(2.0)])
    );
}

#[test]
fn append_both_empty() {
    assert_eq!(eval_str("(append nil nil)"), Value::Nil(Span::default()));
}

#[test]
fn append_does_not_mutate_first_list() {
    assert_eq!(
        eval_str_env(&["(def xs '(1 2))", "(append xs '(3 4))", "xs"]),
        Value::to_cons_list(vec![num(1.0), num(2.0)])
    );
}

#[test]
fn append_second_arg_need_not_be_a_list() {
    // append only requires the first arg to be a proper list - the second
    // becomes the new tail as-is, so this produces an improper list.
    assert_eq!(
        eval_str("(append '(1 2) 3)"),
        cons(num(1.0), cons(num(2.0), num(3.0)))
    );
}

#[test]
fn append_too_few_args_errors() {
    eval_err("(append '(1 2))");
}

#[test]
fn append_first_arg_not_a_list_errors() {
    eval_err("(append 1 '(2 3))");
}

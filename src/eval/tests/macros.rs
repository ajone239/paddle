use super::*;

#[test]
fn defm_basic_expansion() {
    assert_eq!(
        eval_str_env(&vec!["(defm (double x) (list '+ x x))", "(double 5)",]),
        num(10.0)
    );
}

#[test]
fn defm_args_not_evaluated_before_expansion() {
    assert_eq!(
        eval_str_env(&vec![
            "(defm (ignore-arg x) (list 'quote 42))",
            "(ignore-arg undefined-var)",
        ]),
        num(42.0)
    );
}

#[test]
fn defm_when_true() {
    assert_eq!(
        eval_str_env(&vec![
            "(defm (when cond body) (list 'if cond body '()))",
            "(when #t 99)",
        ]),
        num(99.0)
    );
}

#[test]
fn defm_when_false_returns_nil() {
    assert_eq!(
        eval_str_env(&vec![
            "(defm (when cond body) (list 'if cond body '()))",
            "(when #f 99)",
        ]),
        Value::Nil
    );
}

#[test]
fn defm_unless_false_runs_body() {
    assert_eq!(
        eval_str_env(&vec![
            "(defm (unless cond body) (list 'if (list 'not cond) body '()))",
            "(unless #f 7)",
        ]),
        num(7.0)
    );
}

#[test]
fn defm_unless_true_returns_nil() {
    assert_eq!(
        eval_str_env(&vec![
            "(defm (unless cond body) (list 'if (list 'not cond) body '()))",
            "(unless #t 7)",
        ]),
        Value::Nil
    );
}

#[test]
fn define_macro_keyword() {
    assert_eq!(
        eval_str_env(&vec!["(define-macro (inc x) (list '+ x 1))", "(inc 41)",]),
        num(42.0)
    );
}

#[test]
fn macro_used_inside_function() {
    assert_eq!(
        eval_str_env(&vec![
            "(defm (when cond body) (list 'if cond body '()))",
            "(def (safe-inc x) (when (< 0 x) (+ x 1)))",
            "(safe-inc 5)",
        ]),
        num(6.0)
    );
}

#[test]
fn macro_used_inside_function_condition_false() {
    assert_eq!(
        eval_str_env(&vec![
            "(defm (when cond body) (list 'if cond body '()))",
            "(def (safe-inc x) (when (< 0 x) (+ x 1)))",
            "(safe-inc -1)",
        ]),
        Value::Nil
    );
}

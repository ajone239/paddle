use super::*;

#[test]
fn call_immediately() {
    assert_eq!(eval_str("((lambda (x) (* x 2)) 5)"), num(10.0));
}

#[test]
fn no_args() {
    assert_eq!(eval_str("((lambda () 42))"), num(42.0));
}

#[test]
fn multi_arg() {
    assert_eq!(eval_str("((lambda (x y) (+ x y)) 3 4)"), num(7.0));
}

#[test]
fn assign_and_call() {
    assert_eq!(
        eval_str_env(&["(def double (lambda (x) (* x 2)))", "(double 6)"]),
        num(12.0)
    );
}

#[test]
fn multi_body_returns_last() {
    assert_eq!(eval_str("((lambda (x) (+ x 1) (* x 2)) 3)"), num(6.0));
}

#[test]
fn captures_outer_var() {
    assert_eq!(
        eval_str_env(&["(def y 10)", "((lambda (x) (+ x y)) 5)"]),
        num(15.0)
    );
}

#[test]
fn args_do_not_leak() {
    assert_eq!(
        eval_str_env(&["(def x 99)", "((lambda (x) (* x 2)) 3)", "x"]),
        num(99.0)
    );
}

#[test]
fn closure_captures_creation_env() {
    assert_eq!(
        eval_str_env(&[
            "(def (make-adder n) (lambda (x) (+ x n)))",
            "(def add5 (make-adder 5))",
            "(add5 3)"
        ]),
        num(8.0)
    );
}

#[test]
fn higher_order_apply() {
    assert_eq!(
        eval_str_env(&[
            "(def (apply-fn f x) (f x))",
            "(apply-fn (lambda (x) (* x x)) 4)"
        ]),
        num(16.0)
    );
}

#[test]
fn wrong_arity() {
    let err = eval_err("((lambda (x) x) 1 2)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadFunctionArgCount(1, 2))
    );
}

#[test]
fn alternate_syntax_backslash() {
    assert_eq!(eval_str("((.\\  (x) (+ x 1)) 9)"), num(10.0));
}

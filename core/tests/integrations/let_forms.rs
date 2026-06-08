use super::*;

// let

#[test]
fn let_single_binding() {
    assert_eq!(run_macros("(let ((x 5)) x)"), num(5.0));
}

#[test]
fn let_multiple_bindings() {
    assert_eq!(run_macros("(let ((x 3) (y 4)) (+ x y))"), num(7.0));
}

#[test]
fn let_multi_expr_body_returns_last() {
    assert_eq!(run_macros("(let ((x 2)) (+ x 1) (* x 10))"), num(20.0));
}

#[test]
fn let_inner_shadows_outer() {
    assert_eq!(run_macros("(def x 99) (let ((x 1)) x)"), num(1.0));
}

#[test]
fn let_does_not_leak_into_outer_scope() {
    assert_eq!(run_macros("(def x 99) (let ((x 1)) x) x"), num(99.0));
}

#[test]
fn let_bindings_are_parallel() {
    // y is bound to the *outer* x (10), not the let-bound x (20),
    // because let bindings are evaluated before any are in scope.
    assert_eq!(run_macros("(def x 10) (let ((x 20) (y x)) y)"), num(10.0));
}

// let*

#[test]
fn let_star_later_binding_sees_earlier() {
    assert_eq!(run_macros("(let* ((x 3) (y (* x 2))) y)"), num(6.0));
}

#[test]
fn let_star_chain_of_three() {
    assert_eq!(
        run_macros("(let* ((a 1) (b (+ a 1)) (c (+ b 1))) c)"),
        num(3.0)
    );
}

#[test]
fn let_star_inner_shadows_outer() {
    assert_eq!(
        run_macros("(def z 99) (let* ((z 7) (w (* z 2))) w)"),
        num(14.0)
    );
}

//  letrec
#[test]
fn letrec_test() {
    assert_eq!(
        run_macros("(letrec ((f (.\\ (x) (* x 2))) (g (.\\ (y) (+ y 1)))) (def x 2) (f (g x)))"),
        num(6.0)
    );
}

#[test]
fn letrec_mutual_recursion() {
    assert_eq!(
        run_macros(
            "(letrec ((ev? (.\\ (n) (if (= n 0) #t (od? (- n 1))))) \
                              (od? (.\\ (n) (if (= n 0) #f (ev? (- n 1)))))) \
                       (ev? 10))"
        ),
        Value::Bool(true)
    );
}

//  letn
#[test]
fn letn_sum() {
    assert_eq!(
        run_macros("(letn loop ((n 10) (acc 0)) (if (= 0 n) acc (loop (- n 1) (+ n acc))))"),
        num(55.0)
    );
}

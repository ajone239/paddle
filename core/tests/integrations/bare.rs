use super::*;

#[test]
fn arithmetic() {
    assert_eq!(run_bare("(+ (* 3 4) (- 10 4))"), num(18.0));
}

#[test]
fn closure() {
    assert_eq!(
        run_bare(
            "(def (make-adder n) (lambda (x) (+ x n)))
                 (def add10 (make-adder 10))
                 (add10 32)"
        ),
        num(42.0)
    );
}

#[test]
fn list_operations() {
    assert_eq!(run_bare("(car (cdr (list 10 20 30)))"), num(20.0));
}

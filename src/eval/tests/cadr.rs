use super::*;

#[test]
fn car_of_cdr_gives_second_element() {
    assert_eq!(eval_str("(car (cdr '(1 2 3)))"), num(2.0));
}

#[test]
fn nested_cons_car_cdr() {
    // (cons 20 30) => List([20, 30])
    // (cons 10 ...) => List([10, List([20, 30])])
    // cdr         => List([20, 30])
    // car         => 20
    assert_eq!(eval_str("(car (cdr (cons 10 (cons 20 30))))"), num(20.0));
}

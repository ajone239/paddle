use super::*;

#[test]
fn self_recursion_deep() {
    assert_eq!(
        eval_str_env(&[
            "(def (count n) (if (= n 0) 0 (count (- n 1))))",
            "(count 5000)"
        ]),
        num(0.0)
    );
}

#[test]
fn mutual_recursion_deep() {
    assert_eq!(
        eval_str_env(&[
            "(def (even? n) (if (= n 0) #t (odd? (- n 1))))",
            "(def (odd? n) (if (= n 0) #f (even? (- n 1))))",
            "(even? 5000)"
        ]),
        Value::Bool(true)
    );
}

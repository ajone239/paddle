use super::*;

#[test]
fn basic_sum() {
    assert_eq!(
        eval_str_env(&vec![
            "(def (sum xs) (if xs (+ (car xs) (sum (cdr xs))) 0))",
            "(def (t a...) (sum a...))",
            "(t 1 2 3 4)",
        ]),
        num(10.0)
    );
}

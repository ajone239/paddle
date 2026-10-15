use super::*;

#[test]
fn define_variadic() {
    assert_eq!(
        eval_str_env(&[
            "(def (sum xs)
                (if xs
                 (+ (car xs) (sum (cdr xs)))
                 0))",
            "(def (f x...) (sum x...))",
            "(f 1 2 3)"
        ]),
        num(6.0)
    );
}

#[test]
fn define_variadic_lambda() {
    assert_eq!(
        eval_str_env(&[
            "(def (sum xs)
                (if xs
                 (+ (car xs) (sum (cdr xs)))
                 0))",
            "((.\\ (x...) (sum x...)) 1 2 3)"
        ]),
        num(6.0)
    );
}

#[test]
fn define_variadic_eval() {
    assert_eq!(
        eval_str_env(&[
            "(def (sum xs)
                (if xs
                 (+ (car xs) (sum (cdr xs)))
                 0))",
            "(def (f x...) (sum x...))",
            "(f 1 2 (+ 1 2))"
        ]),
        num(6.0)
    );
}

#[test]
fn define_variadic_lambda_eval() {
    assert_eq!(
        eval_str_env(&[
            "(def (sum xs)
                (if xs
                 (+ (car xs) (sum (cdr xs)))
                 0))",
            "((.\\ (x...) (sum x...)) 1 2 (+ 1 2))"
        ]),
        num(6.0)
    );
}

#[test]
fn define_variadic_macro() {
    assert_eq!(
        eval_str_env(&[
            "
(defmacro (cond pairs...)
    (define (cond2 pairs)
      (if pairs
       `(if ,(car (car pairs))
            ,(car (cdr (car pairs)))
            ,(cond2 (cdr pairs)))
        '()))
    (cond2 pairs...))",
            "
(def (f v)
    (cond
        ((= 0 v) 'zero)
        ((= 0 (% v 2)) 'even)
        (#t 'odd)))",
            "(list (f 0) (f 1) (f 4))"
        ]),
        Value::to_cons_list(vec![sym("zero"), sym("odd"), sym("even"),])
    );
}

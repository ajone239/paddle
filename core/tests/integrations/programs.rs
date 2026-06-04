use super::*;

#[test]
fn process_returns_a_value_per_expression() {
    let env = Rc::new(RefCell::new(Env::default()));
    let results = process("(+ 1 2) (* 3 4) (- 10 5)", env).unwrap();
    assert_eq!(results, vec![num(3.0), num(12.0), num(5.0)]);
}

#[test]
fn definition_contributes_nil_to_result_list() {
    let env = Rc::new(RefCell::new(Env::default()));
    let results = process("(def x 42)", env).unwrap();
    assert_eq!(results, vec![Value::Nil]);
}

#[test]
fn definitions_visible_to_later_expressions_in_same_call() {
    assert_eq!(run_bare("(def (double x) (* x 2)) (double 21)"), num(42.0));
}

#[test]
fn env_persists_across_separate_process_calls() {
    let env = Rc::new(RefCell::new(Env::default()));
    process("(def (inc x) (+ x 1))", env.clone()).expect("first call failed");
    let mut r = process("(inc 41)", env).expect("second call failed");
    assert_eq!(r.pop().unwrap(), num(42.0));
}

#[test]
fn chained_function_definitions() {
    assert_eq!(
        run("(def (double x) (* x 2))
                 (def (quad x) (double (double x)))
                 (quad 5)"),
        num(20.0)
    );
}

#[test]
fn recursive_factorial_inline() {
    assert_eq!(
        run("(def (fact n) (if (< n 1) 1 (* n (fact (- n 1))))) (fact 7)"),
        num(5040.0)
    );
}

#[test]
fn higher_order_inline() {
    assert_eq!(
        run("(def (apply-twice f x) (f (f x)))
                 (apply-twice (lambda (x) (+ x 3)) 10)"),
        num(16.0)
    );
}

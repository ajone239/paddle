use super::*;
use paddle_core::eval::{EvalError, value::Form};

#[test]
fn undefined_symbol_bubbles_up() {
    let err = run_err("totally-undefined-symbol");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::SymbolUndefined(
            "totally-undefined-symbol".into()
        ))
    );
}

#[test]
fn wrong_arg_count_bubbles_up() {
    let err = run_err("(def (f x) (+ x 1)) (f 1 2)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadFunctionArgCount(1))
    );
}

#[test]
fn bad_lambda_missing_body_bubbles_up() {
    let err = run_err("(lambda (x))");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadCallableBodyArgs(Form::Lambda))
    );
}

#[test]
fn error_in_later_expression_stops_program() {
    // First expression succeeds; second expression should fail
    let env = Rc::new(RefCell::new(Env::default()));
    process(STD_LIB, env.clone()).expect("stdlib failed");
    let result = process("(+ 1 2) undefined-sym", env.clone());
    assert!(result.is_err());
}

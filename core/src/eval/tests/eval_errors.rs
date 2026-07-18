use super::*;

#[test]
fn undefined_symbol() {
    let err = eval_err("x");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::SymbolUndefined("x".into(), Span::default()))
    );
}

#[test]
fn bad_define_args() {
    let err = eval_err("(def x)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadDefineArgs(Span::default()))
    );
}

#[test]
fn bad_lambda_args() {
    let err = eval_err("(lambda (x))");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadCallableBodyArgs(
            Form::Lambda,
            Span::default()
        ))
    );
}

#[test]
fn bad_lambda_args_list() {
    let err = eval_err("(lambda x x)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadCallableArgs(Form::Lambda, Span::default()))
    );
}

#[test]
fn bad_lambda_args_list_type() {
    let err = eval_err("(lambda (1) x)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadCallableArgsListType(
            Form::Lambda,
            Span::default()
        ))
    );
}

#[test]
fn bad_define_head() {
    let err = eval_err("(def 5 x)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadDefineHead(Span::default()))
    );
}

#[test]
fn bad_define_function_head() {
    let err = eval_err("(def () x)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadDefineHead(Span::default()))
    );
}

#[test]
fn bad_define_function_head_types() {
    let err = eval_err("(def (1 x) x)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadCallableArgsListType(
            Form::Define,
            Span::default()
        ))
    );
}

#[test]
fn if_no_else_true() {
    let err = eval_err("(if #t 42)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadIfArgs(Span::default()))
    );
}

#[test]
fn if_no_else_false() {
    let err = eval_err("(if #f 42)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadIfArgs(Span::default()))
    );
}

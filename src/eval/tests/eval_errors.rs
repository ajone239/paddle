use super::*;

#[test]
fn bad_define_args() {
    let err = eval_err("(def x)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadDefineArgs)
    );
}

#[test]
fn bad_lambda_args() {
    let err = eval_err("(lambda (x))");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadLambdaArgs)
    );
}

#[test]
fn bad_lambda_args_list() {
    let err = eval_err("(lambda x x)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadLambdaArgsList)
    );
}

#[test]
fn bad_lambda_args_list_type() {
    let err = eval_err("(lambda (1) x)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadLambdaArgsListType)
    );
}

#[test]
fn bad_define_head() {
    let err = eval_err("(def 5 x)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadDefineHead)
    );
}

#[test]
fn bad_define_function_head() {
    let err = eval_err("(def () x)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadDefineFunctionHead)
    );
}

#[test]
fn bad_define_function_head_types() {
    let err = eval_err("(def (1 x) x)");
    assert_eq!(
        err.downcast_ref::<EvalError>(),
        Some(&EvalError::BadDefineFunctionHeadTypes)
    );
}

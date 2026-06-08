mod env;
mod eval;
mod lower;
#[cfg(test)]
mod tests;
pub mod value;

pub use env::Env;
pub use eval::eval;
pub use lower::lower;

use thiserror::Error;

use crate::eval::value::Form;

#[derive(Debug, PartialEq, Error)]
pub enum EvalError {
    #[error("Require takes 2 arguments got {0}")]
    BadRequireArgCount(usize),
    #[error("Require takes strings or symbols as args")]
    BadRequireArgs,
    #[error("Too few arguments were provided to the set! statement")]
    BadSetBangArgs,
    #[error("Too few arguments were provided to the define statement")]
    BadDefineArgs,
    #[error("Too few arguments were provided to the if statement")]
    BadIfArgs,
    #[error("Too few arguments were provided to the lambda statement")]
    BadLambdaArgs,
    #[error("A list is required for lambda args")]
    BadLambdaArgsList,
    #[error("Symbol [{0}] is undefined in current env.")]
    SymbolUndefined(String),
    #[error("Symbol or list expected.")]
    BadDefineHead,
    #[error("Lambda function args list must only be symbols")]
    BadLambdaArgsListType,
    #[error("Function expected {0} args.")]
    BadFunctionArgCount(usize),
    #[error("Function definition requires atleast a function name.")]
    BadDefineFunctionHead,
    #[error("Function definition head may only contain symbols.")]
    BadDefineFunctionHeadTypes,
    #[error("Progn body must have entries")]
    EmptyPrognBody,
    #[error("Unquoute called outside of a quasiquote context")]
    UnquoteOutsideQuasi,
    #[error("Variadic args must come last.")]
    VariadicArgsMustBeLast,
    #[error("No args provided for callable: {0:?}")]
    BadCallableArgs(Form),
    #[error("bad types for callable: {0:?}")]
    BadCallableArgsListType(Form),
    #[error("Need name for named callable: {0:?}")]
    BadCallableHead(Form),
    #[error("No body for callable: {0:?}")]
    BadCallableBodyArgs(Form),
}

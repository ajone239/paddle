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

#[derive(Debug, PartialEq, Error)]
pub enum EvalError {
    #[error("Require takes 2 arguments got {0}")]
    BadRequireArgCount(usize),
    #[error("Require takes strings or symbols as args")]
    BadRequireArgs,
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
    #[error("Function expected {0} args but got {1}.")]
    BadFunctionArgCount(usize, usize),
    #[error("Function definition requires atleast a function name.")]
    BadDefineFunctionHead,
    #[error("Function definition head may only contain symbols.")]
    BadDefineFunctionHeadTypes,
    #[error("PrognBodyMustHaveEntries")]
    EmptyPrognBody,
    #[error("Unquoute called outside of a quasiquote context")]
    UnquoteOutsideQuasi,
}

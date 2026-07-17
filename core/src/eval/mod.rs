mod builtins;
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

use crate::{eval::value::Form, lexer::Span};

#[derive(Debug, PartialEq, Error)]
pub enum EvalError {
    #[error("Require takes 2 arguments got {0} as {1}")]
    BadRequireArgCount(usize, Span),
    #[error("Require takes strings or symbols as args as {0}")]
    BadRequireArgs(Span),
    #[error("Too few arguments were provided to the set! statement at {0}")]
    BadSetBangArgs(Span),
    #[error("Too few arguments were provided to the define statement at {0}")]
    BadDefineArgs(Span),
    #[error("Too few arguments were provided to the if statement at {0}")]
    BadIfArgs(Span),
    #[error("Too few arguments were provided to the lambda statement at {0}")]
    BadLambdaArgs(Span),
    #[error("A list is required for lambda args at {0}")]
    BadLambdaArgsList(Span),
    #[error("Symbol [{0}] is undefined in current env. at {1}")]
    SymbolUndefined(String, Span),
    #[error("Symbol or list expected. at {0}")]
    BadDefineHead(Span),
    #[error("Lambda function args list must only be symbols at {0}")]
    BadLambdaArgsListType(Span),
    #[error("Function expected {0} args. at {1}")]
    BadFunctionArgCount(usize, Span),
    #[error("Function definition requires atleast a function name. at {0}")]
    BadDefineFunctionHead(Span),
    #[error("Function definition head may only contain symbols. at {0}")]
    BadDefineFunctionHeadTypes(Span),
    #[error("Progn body must have entries at {0}")]
    EmptyPrognBody(Span),
    #[error("Unquoute called outside of a quasiquote context at {0}")]
    UnquoteOutsideQuasi(Span),
    #[error("Variadic args must come last. at {0}")]
    VariadicArgsMustBeLast(Span),
    #[error("No args provided for callable: {0:?} at {1}")]
    BadCallableArgs(Form, Span),
    #[error("bad types for callable: {0:?} at {1}")]
    BadCallableArgsListType(Form, Span),
    #[error("Need name for named callable: {0:?} at {1}")]
    BadCallableHead(Form, Span),
    #[error("No body for callable: {0:?} at {1}")]
    BadCallableBodyArgs(Form, Span),
}

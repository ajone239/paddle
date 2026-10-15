mod builtins;
mod store;

use thiserror::Error;

pub use store::Env;

#[derive(Debug, PartialEq, Error)]
pub enum BuiltinError {
    #[error("Not: Expected 1 argument.")]
    WrongNotArgCount,
    #[error("Cons: Expected 1 argument.")]
    WrongConsArgCount,
    #[error("Car: Expected 1 argument.")]
    WrongCarArgCount,
    #[error("Car: Must be applied to a list.")]
    WrongCarArgType,
    #[error("Cdr: Expected 1 argument.")]
    WrongCdrArgCount,
    #[error("Cdr: Must be applied to a list.")]
    WrongCdrArgType,
    #[error("Cdr: Cannot be applied to an empty list.")]
    CdrOnEmptyList,
    #[error("Expected Number for arithmetic builtin.")]
    ExpectedNumArg,
    #[error("Minus: initial argument required.")]
    NoInitforMinus,
    #[error("LessThan: Expected Numbers")]
    BadLtArgTypes,
    #[error("Div: initial argument required.")]
    NoInitforDiv,
    #[error("Car: Cannot be applied to an empty list.")]
    CarOnEmptyList,
    #[error("Modulo: Expected numbers")]
    BadModArgTypes,
    #[error("Modulo: Expected 2 arguments")]
    BadModArgCount,
    #[error("Eq: Expected 2 arguments")]
    BadEqArgCount,
}

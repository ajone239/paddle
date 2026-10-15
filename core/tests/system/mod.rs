mod arithmetic;
mod atoms;
mod builtin_errors;
mod cadr;
mod car;
mod cdr;
mod char_tests;
mod cons;
mod empty;
mod env;
mod eq;
mod eval_errors;
mod lambda;
mod list;
mod macros;
mod predicates;
mod quote;
mod set_bang;
mod splice;
mod string_tests;
mod tco;
mod variadic;

use paddle_core::test_utils::*;
use paddle_core::{
    env::BuiltinError,
    span::Span,
    value::{Form, Value},
};

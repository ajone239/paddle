pub mod cursor;
pub mod eval;
pub mod lexer;
pub mod parser;
pub mod span;

pub fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

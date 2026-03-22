use anyhow::Result;
use jethe::{lexer, parser};

fn main() -> Result<()> {
    let test_code = "(+ 1 2)";

    let tokens = lexer::lex(test_code);

    let (ast, rest) = parser::parse_expr(&tokens)?;

    println!("{:?}", ast);
    println!("{:?}", rest);

    Ok(())
}

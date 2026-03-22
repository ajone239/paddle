use std::io::{Write, stdin, stdout};

use anyhow::Result;
use jethe::{eval::eval, lexer, parser};

fn main() -> Result<()> {
    let stdin = stdin();

    print!("> ");
    stdout().flush().unwrap();
    for line in stdin.lines() {
        let line = line?;

        let tokens = lexer::lex(&line);

        let (ast, _) = parser::parse_expr(&tokens)?;

        let val = eval(&ast);

        println!("{:?}", val);
        print!("> ");
        stdout().flush().unwrap();
    }

    Ok(())
}

use std::io::{Write, stdin, stdout};

use anyhow::Result;
use jethe::{
    env::Env,
    eval::{eval, lower},
    lexer, parser,
};

fn main() -> Result<()> {
    let stdin = stdin();

    print!("> ");
    stdout().flush().unwrap();

    let mut input = String::new();

    let mut env = Env::default();

    for line in stdin.lines() {
        let line = line?;

        if input.is_empty() {
            match line.as_str() {
                ":env" => {
                    println!("{:#?}", env);
                    prompt(0);
                    continue;
                }
                _ => {}
            }
        }

        input += &line;

        let pcount = count_paren(&input);

        match pcount {
            c if c < 0 => println!("Err: bad paren structure!"),
            c if c > 0 => {
                prompt(pcount as usize);
                continue;
            }
            _ => {
                let tokens = lexer::lex(&line);
                let (ast, _) = parser::parse_expr(&tokens)?;
                let expr = lower(&ast);
                let val = eval(&expr, &mut env);

                println!("{:?}", val);
            }
        }

        input.clear();
        prompt(0);
    }

    Ok(())
}

fn prompt(indent: usize) {
    if indent == 0 {
        print!("> ");
    } else {
        let p = "  ".repeat(indent);
        print!("* {}", p);
    }

    stdout().flush().unwrap();
}

fn count_paren(line: &str) -> i32 {
    if line.is_empty() {
        return 0;
    }

    line.chars()
        .map(|c| match c {
            '(' => 1,
            ')' => -1,
            _ => 0,
        })
        .sum()
}

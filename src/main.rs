use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use rustyline::{DefaultEditor, error::ReadlineError};

use jethe::{
    eval::{env::Env, eval, lower, value::Value},
    lexer, parser,
};

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;

    let mut input = String::new();

    let env = Rc::new(RefCell::new(Env::default()));

    loop {
        let pcount = count_paren(&input);
        let prompt = make_prompt(pcount as usize);

        match rl.readline(&prompt) {
            Ok(line) => {
                let line = line.trim();

                if input.is_empty() && ":env" == line {
                    println!("{:#?}", env.borrow());
                    rl.add_history_entry(line)?;
                    continue;
                }

                input += line;

                let pcount = count_paren(&input);

                match pcount {
                    c if c < 0 => println!("Err: bad paren structure!"),
                    c if c > 0 => continue,
                    _ => {
                        rl.add_history_entry(&input)?;
                        let val = lpe(&input, env.clone());
                        println!("{:?}", val);
                    }
                }

                input.clear();
            }
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => break,
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

fn lpe(input: &str, env: Rc<RefCell<Env>>) -> Result<Value> {
    let tokens = lexer::lex(input);
    let (ast, _) = parser::parse_expr(&tokens)?;
    let expr = lower(&ast);
    eval(&expr, env)
}

fn make_prompt(indent: usize) -> String {
    if indent == 0 {
        "> ".to_string()
    } else {
        let p = "  ".repeat(indent);
        format!("* {}", p)
    }
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

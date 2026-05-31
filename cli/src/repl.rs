use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use rustyline::{DefaultEditor, error::ReadlineError};

use paddle_core::{
    cursor::{Cursor, count_paren, display_result, is_ready_to_process},
    eval::Env,
    lexer,
};

pub fn run_repl(env: Rc<RefCell<Env>>) -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    let mut input = String::new();

    loop {
        let pcount = count_paren(&input);
        let prompt = make_prompt(pcount as usize);

        let line = match rl.readline(&prompt) {
            Ok(line) => line,
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => break,
            Err(e) => return Err(e.into()),
        };

        let line = line.trim();

        if input.is_empty() && handle_repl_cmd(env.clone(), line) {
            rl.add_history_entry(line)?;
            continue;
        }

        input += line;

        let ready = match is_ready_to_process(&input) {
            Ok(ready) => ready,
            Err(err) => {
                println!("ERROR: {}", err);
                input.clear();
                continue;
            }
        };

        if !ready {
            continue;
        }

        rl.add_history_entry(&input)?;

        let lexed = lexer::lex(&input);
        let cursor = Cursor::new(&lexed, env.clone());

        for res in cursor {
            display_result(res);
        }

        input.clear();
    }

    Ok(())
}

fn handle_repl_cmd(env: Rc<RefCell<Env>>, line: &str) -> bool {
    match line {
        ":env" => {
            env.borrow().dump();
            true
        }
        _ => false,
    }
}

fn make_prompt(indent: usize) -> String {
    if indent == 0 {
        "> ".to_string()
    } else {
        let p = "  ".repeat(indent);
        format!("* {}", p)
    }
}

use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use rustyline::{DefaultEditor, error::ReadlineError};

use paddle_core::{
    cursor::{Cursor, count_paren, display_result, is_ready_to_process},
    eval::Env,
    lexer,
    span::intern,
};

pub fn run_repl(env: Rc<RefCell<Env>>) -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    let mut input = String::new();
    let repl_file_id = intern("<REPL>".to_string());

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

        let lexed = lexer::lex(&input, repl_file_id);
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
        ":e" | ":env" => {
            env.borrow().dump();
            true
        }
        ":v" | ":version" => {
            println!("REPL: {}", env!("CARGO_PKG_VERSION"));
            println!("Core: {}", paddle_core::core_version());
            true
        }
        s if s.starts_with(":env_get") => {
            let values: Vec<&str> = line.split_whitespace().collect();
            if values.len() != 2 {
                println!("Usage: :env_get <value_name>");
                return true;
            }
            let val = values[1];

            match env.borrow().resolve(val) {
                Some(value) => {
                    println!("Value [{val}]");
                    println!("{}", value.dump());
                }
                None => {
                    println!("Value [{val}] not found in current env");
                }
            }

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

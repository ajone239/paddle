use std::{cell::RefCell, fs::read_to_string, rc::Rc};

use anyhow::Result;
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::{
    cursor::{count_paren, display_results, is_ready_to_process, process},
    eval::Env,
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

        if input.is_empty() {
            if handle_repl_cmd(&env, line) {
                rl.add_history_entry(line)?;
                continue;
            }
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
        let res = process(&input, env.clone());
        display_results(res);

        input.clear();
    }

    Ok(())
}

fn handle_repl_cmd(env: &Rc<RefCell<Env>>, line: &str) -> bool {
    match line {
        ":env" => {
            env.borrow().dump();
            true
        }
        s if s.starts_with(":require ") => {
            s.split_whitespace()
                .skip(1)
                .for_each(|f| match read_to_string(f) {
                    Ok(contents) => {
                        if let Err(err) = process(&contents, env.clone()) {
                            println!("Problem reading {}: {:?}", f, err);
                        }
                    }
                    Err(err) => println!("Problem reading {}: {:?}", f, err),
                });
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

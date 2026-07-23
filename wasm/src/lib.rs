use std::{cell::RefCell, rc::Rc};

use paddle_core::{
    cursor::Cursor,
    eval::{Env, value::Value},
    lexer,
};
use wasm_bindgen::prelude::*;

/*
 * Le plan
 *
 * - make a result type
 *     + maybe box?
 * - store them in a vec
 * - expose the vec with memory.js
 * - render them
 * - clear on render
 */

#[wasm_bindgen]
pub enum RunResult {
    Good(String),
    Bad(String),
}

#[wasm_bindgen]
impl RunResult {
    fn len(&self) -> usize {
        match self {
            RunResult::Good(s) => s.len(),
            RunResult::Bad(s) => s.len(),
        }
    }
}

#[wasm_bindgen]
pub struct Runner {
    env: Rc<RefCell<Env>>,
    results: Vec<RunResult>,
}

#[wasm_bindgen]
impl Runner {
    pub fn new() -> Self {
        let env = Env::with_stdlib().expect("shouldn't fail");

        Self {
            env,
            results: vec![],
        }
    }

    pub fn results_len(&self) -> usize {
        self.results.len()
    }

    pub fn results_ptr(&self) -> *const RunResult {
        self.results.as_ptr()
    }

    pub fn run_code(&mut self, input: &str) {
        let lexed = lexer::lex(&input);

        let cursor = Cursor::new(&lexed, self.env.clone());

        for res in cursor {
            let rr = match res {
                Err(err) => RunResult::Bad(format!("ERROR: {:?}", err)),
                Ok(Value::NoPrint) => RunResult::Good("{...}".to_string()),
                Ok(val) => RunResult::Good(format!("{}", val)),
            };

            self.results.push(rr);
        }
    }
}

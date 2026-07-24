use std::{cell::RefCell, rc::Rc};

use js_sys::Function;
use paddle_core::{
    cursor::Cursor,
    eval::{Env, value::Value},
    lexer,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub enum Tag {
    Err,
    Print,
    Ok,
    Source,
}

#[wasm_bindgen]
pub struct Runner {
    env: Rc<RefCell<Env>>,
}

#[wasm_bindgen]
impl Runner {
    pub fn new() -> Self {
        let env = Env::with_stdlib().expect("shouldn't fail");

        Self { env }
    }

    pub fn run_code(&self, input: &str, callback: Function) {
        let lexed = lexer::lex(&input);

        let cursor = Cursor::new(&lexed, self.env.clone());

        for res in cursor {
            let (tag, content) = match res {
                Err(err) => (Tag::Err, format!("ERROR: {:?}", err)),
                Ok(Value::NoPrint) => (Tag::Print, "{...}".to_string()),
                Ok(val) => (Tag::Ok, format!("{}", val)),
            };

            let _ = callback.call2(&JsValue::NULL, &tag.into(), &content.into());
        }
    }
}

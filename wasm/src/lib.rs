use paddle_core::{
    cursor::Cursor,
    eval::{Env, value::Value},
    lexer,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
pub fn run_code(input: &str) {
    let env = Env::with_stdlib().expect("shouldn't fail");
    let lexed = lexer::lex(&input);

    let cursor = Cursor::new(&lexed, env);

    for res in cursor {
        match res {
            Err(err) => alert(&format!("ERROR: {:?}", err)),
            Ok(Value::NoPrint) => {}
            Ok(val) => alert(&format!("{}", val)),
        }
    }
}

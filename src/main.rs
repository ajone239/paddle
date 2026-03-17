use jethe::lexer;

fn main() {
    let test_code = "(+ 1 2)";

    let tokens = lexer::lex(test_code);

    println!("{:?}", tokens);
}

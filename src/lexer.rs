use crate::split_keeping_delimiter::SplitKeepingDelimiterExt;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    LeftParen,
    Symbol(&'a str),
    RigthParen,
}

impl<'a> Token<'a> {
    fn from_str(s: &'a str) -> Self {
        match s {
            "(" => Self::LeftParen,
            ")" => Self::RigthParen,
            _ => Self::Symbol(s),
        }
    }
}

pub fn lex<'a>(code: &'a str) -> Vec<Token<'a>> {
    code.split_keeping_delimiter(&[' ', '\n', '(', ')'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| Token::from_str(s))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sym(s: &str) -> Token<'_> {
        Token::Symbol(s)
    }

    // --- basic structure ---

    #[test]
    fn empty_input() {
        assert_eq!(lex(""), vec![]);
    }

    #[test]
    fn only_whitespace() {
        assert_eq!(lex("   \n  "), vec![]);
    }

    #[test]
    fn single_number() {
        assert_eq!(lex("42"), vec![Token::Symbol("42")]);
    }

    #[test]
    fn single_symbol() {
        assert_eq!(lex("foo"), vec![sym("foo")]);
    }

    #[test]
    fn empty_parens() {
        assert_eq!(lex("()"), vec![Token::LeftParen, Token::RigthParen]);
    }

    // --- simple expressions ---

    #[test]
    fn simple_addition() {
        assert_eq!(
            lex("(+ 1 2)"),
            vec![
                Token::LeftParen,
                sym("+"),
                Token::Symbol("1"),
                Token::Symbol("2"),
                Token::RigthParen,
            ]
        );
    }

    #[test]
    fn define_expression() {
        assert_eq!(
            lex("(define x 10)"),
            vec![
                Token::LeftParen,
                sym("define"),
                sym("x"),
                Token::Symbol("10"),
                Token::RigthParen,
            ]
        );
    }

    // --- whitespace handling ---

    #[test]
    fn extra_spaces_between_tokens() {
        assert_eq!(
            lex("(+   1   2)"),
            vec![
                Token::LeftParen,
                sym("+"),
                Token::Symbol("1"),
                Token::Symbol("2"),
                Token::RigthParen,
            ]
        );
    }

    #[test]
    fn newlines_between_tokens() {
        assert_eq!(
            lex("(+\n1\n2)"),
            vec![
                Token::LeftParen,
                sym("+"),
                Token::Symbol("1"),
                Token::Symbol("2"),
                Token::RigthParen,
            ]
        );
    }

    // --- nesting ---

    #[test]
    fn nested_expression() {
        assert_eq!(
            lex("(+ (- 3 1) 2)"),
            vec![
                Token::LeftParen,
                sym("+"),
                Token::LeftParen,
                sym("-"),
                Token::Symbol("3"),
                Token::Symbol("1"),
                Token::RigthParen,
                Token::Symbol("2"),
                Token::RigthParen,
            ]
        );
    }

    #[test]
    fn deeply_nested() {
        assert_eq!(
            lex("(a (b (c)))"),
            vec![
                Token::LeftParen,
                sym("a"),
                Token::LeftParen,
                sym("b"),
                Token::LeftParen,
                sym("c"),
                Token::RigthParen,
                Token::RigthParen,
                Token::RigthParen,
            ]
        );
    }

    // --- number formats ---

    #[test]
    fn float_number() {
        assert_eq!(lex("3.14"), vec![Token::Symbol("3.14")]);
    }

    #[test]
    fn negative_number() {
        assert_eq!(lex("-7"), vec![Token::Symbol("-7")]);
    }

    #[test]
    fn negative_float() {
        assert_eq!(lex("-0.5"), vec![Token::Symbol("-0.5")]);
    }

    // --- symbols ---

    #[test]
    fn operator_symbols() {
        for op in ["+", "-", "*", "/", "=", "<", ">", "<=", ">="] {
            assert_eq!(lex(op), vec![sym(op)], "operator: {op}");
        }
    }

    #[test]
    fn multi_char_symbol() {
        assert_eq!(lex("lambda"), vec![sym("lambda")]);
    }

    #[test]
    fn symbol_with_hyphen() {
        assert_eq!(lex("my-var"), vec![sym("my-var")]);
    }

    #[test]
    fn symbol_with_question_mark() {
        assert_eq!(lex("nil?"), vec![sym("nil?")]);
    }

    // --- known failures: whacky whitespace ---
    // tabs and carriage returns are not treated as whitespace delimiters,
    // so they get absorbed into adjacent tokens as garbage characters.

    #[test]
    fn tab_between_tokens() {
        // "\t" should be whitespace, not part of the symbol "+\t"1"
        assert_eq!(
            lex("(+\t1\t2)"),
            vec![
                Token::LeftParen,
                sym("+"),
                Token::Symbol("1"),
                Token::Symbol("2"),
                Token::RigthParen,
            ]
        );
    }

    #[test]
    fn carriage_return_between_tokens() {
        // "\r\n" Windows line endings should be treated as whitespace
        assert_eq!(
            lex("(+\r\n1\r\n2)"),
            vec![
                Token::LeftParen,
                sym("+"),
                Token::Symbol("1"),
                Token::Symbol("2"),
                Token::RigthParen,
            ]
        );
    }

    #[test]
    fn only_tabs_and_carriage_returns() {
        assert_eq!(lex("\t\t\r\n\t"), vec![]);
    }

    #[test]
    fn mixed_whitespace_between_tokens() {
        assert_eq!(
            lex("(+  \t  1)"),
            vec![
                Token::LeftParen,
                sym("+"),
                Token::Symbol("1"),
                Token::RigthParen,
            ]
        );
    }

    // --- known failures: string literals ---
    // spaces inside quotes are treated as delimiters, shattering the string
    // into separate tokens instead of one Symbol containing the whole literal.

    #[test]
    fn string_literal_no_spaces() {
        // even without spaces, the quotes are still part of the symbol
        assert_eq!(lex("\"hello\""), vec![sym("\"hello\"")]);
    }

    #[test]
    fn string_literal_with_spaces() {
        // the space causes "hello world" to be split into three tokens
        assert_eq!(
            lex("(print \"hello world\")"),
            vec![
                Token::LeftParen,
                sym("print"),
                sym("\"hello world\""),
                Token::RigthParen,
            ]
        );
    }

    #[test]
    fn empty_string_literal() {
        assert_eq!(lex("\"\""), vec![sym("\"\"")]);
    }

    #[test]
    fn string_containing_parens() {
        // parens inside a string should not produce LeftParen/RightParen tokens
        assert_eq!(lex("\"(not a paren)\""), vec![sym("\"(not a paren)\"")]);
    }

    // --- multiple top-level forms ---

    #[test]
    fn two_top_level_expressions() {
        assert_eq!(
            lex("(+ 1 2) (- 3 4)"),
            vec![
                Token::LeftParen,
                sym("+"),
                Token::Symbol("1"),
                Token::Symbol("2"),
                Token::RigthParen,
                Token::LeftParen,
                sym("-"),
                Token::Symbol("3"),
                Token::Symbol("4"),
                Token::RigthParen,
            ]
        );
    }
}

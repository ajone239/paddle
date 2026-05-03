#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum TokenKind<'a> {
    LeftParen,
    RightParen,
    Quote,
    QuasiQuote,
    UnQuote,
    Symbol(&'a str),
}

pub fn lex<'a>(code: &'a str) -> Vec<Token<'a>> {
    let mut tokens = Vec::with_capacity(code.len() / 4);

    let mut line = 0;
    let mut column = 0;
    let mut token_start_col = 0;
    let mut last = 0;
    let mut building_string = false;
    let mut escape = false;
    let mut comment = false;

    for (offset, c) in code.char_indices() {
        column += 1;

        if escape {
            escape = false;
            continue;
        }

        let (buf_end, next_kind) = match c {
            '"' => {
                if !building_string {
                    building_string = true;
                    continue;
                }
                building_string = false;
                (offset + 1, None)
            }
            '\\' if building_string => {
                escape = true;
                continue;
            }
            _ if building_string => continue,
            '\n' => {
                line += 1;
                column = 0;
                comment = false;
                (offset, None)
            }
            _ if comment => (offset, None),
            '(' => (offset, Some(TokenKind::LeftParen)),
            ')' => (offset, Some(TokenKind::RightParen)),
            '\'' => (offset, Some(TokenKind::Quote)),
            '`' => (offset, Some(TokenKind::QuasiQuote)),
            ',' => (offset, Some(TokenKind::UnQuote)),
            ';' => {
                comment = true;
                (offset, None)
            }
            c if c.is_whitespace() => (offset, None),
            _ => {
                if last == offset {
                    token_start_col = column;
                }
                continue;
            }
        };

        // pinch off buffer
        if last != buf_end {
            let token = Token {
                kind: TokenKind::Symbol(&code[last..buf_end]),
                span: Span {
                    line,
                    column: token_start_col,
                },
            };
            tokens.push(token);
        }

        // grab the token
        if let Some(kind) = next_kind {
            let token = Token {
                kind,
                span: Span { line, column },
            };
            tokens.push(token);
        }

        last = offset + c.len_utf8();
    }

    // pinch off what is left
    if last != code.len() {
        let token = Token {
            kind: TokenKind::Symbol(&code[last..]),
            span: Span {
                line,
                column: token_start_col,
            },
        };
        tokens.push(token);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(tokens: Vec<Token>) -> Vec<TokenKind> {
        tokens.into_iter().map(|t| t.kind).collect()
    }

    fn tok(kind: TokenKind, line: usize, column: usize) -> Token {
        Token {
            kind,
            span: Span { line, column },
        }
    }

    fn sym(s: &str) -> TokenKind<'_> {
        TokenKind::Symbol(s)
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
        assert_eq!(kinds(lex("42")), vec![sym("42")]);
    }

    #[test]
    fn single_symbol() {
        assert_eq!(kinds(lex("foo")), vec![sym("foo")]);
    }

    #[test]
    fn empty_parens() {
        assert_eq!(
            kinds(lex("()")),
            vec![TokenKind::LeftParen, TokenKind::RightParen]
        );
    }

    // --- simple expressions ---

    #[test]
    fn simple_addition() {
        assert_eq!(
            kinds(lex("(+ 1 2)")),
            vec![
                TokenKind::LeftParen,
                sym("+"),
                sym("1"),
                sym("2"),
                TokenKind::RightParen
            ]
        );
    }

    #[test]
    fn define_expression() {
        assert_eq!(
            kinds(lex("(define x 10)")),
            vec![
                TokenKind::LeftParen,
                sym("define"),
                sym("x"),
                sym("10"),
                TokenKind::RightParen
            ]
        );
    }

    // --- whitespace handling ---

    #[test]
    fn extra_spaces_between_tokens() {
        assert_eq!(
            kinds(lex("(+   1   2)")),
            vec![
                TokenKind::LeftParen,
                sym("+"),
                sym("1"),
                sym("2"),
                TokenKind::RightParen
            ]
        );
    }

    #[test]
    fn newlines_between_tokens() {
        assert_eq!(
            kinds(lex("(+\n1\n2)")),
            vec![
                TokenKind::LeftParen,
                sym("+"),
                sym("1"),
                sym("2"),
                TokenKind::RightParen
            ]
        );
    }

    // --- nesting ---

    #[test]
    fn nested_expression() {
        assert_eq!(
            kinds(lex("(+ (- 3 1) 2)")),
            vec![
                TokenKind::LeftParen,
                sym("+"),
                TokenKind::LeftParen,
                sym("-"),
                sym("3"),
                sym("1"),
                TokenKind::RightParen,
                sym("2"),
                TokenKind::RightParen,
            ]
        );
    }

    #[test]
    fn deeply_nested() {
        assert_eq!(
            kinds(lex("(a (b (c)))")),
            vec![
                TokenKind::LeftParen,
                sym("a"),
                TokenKind::LeftParen,
                sym("b"),
                TokenKind::LeftParen,
                sym("c"),
                TokenKind::RightParen,
                TokenKind::RightParen,
                TokenKind::RightParen,
            ]
        );
    }

    // --- number formats ---

    #[test]
    fn float_number() {
        assert_eq!(kinds(lex("3.14")), vec![sym("3.14")]);
    }

    #[test]
    fn negative_number() {
        assert_eq!(kinds(lex("-7")), vec![sym("-7")]);
    }

    #[test]
    fn negative_float() {
        assert_eq!(kinds(lex("-0.5")), vec![sym("-0.5")]);
    }

    // --- symbols ---

    #[test]
    fn operator_symbols() {
        for op in ["+", "-", "*", "/", "=", "<", ">", "<=", ">="] {
            assert_eq!(kinds(lex(op)), vec![sym(op)], "operator: {op}");
        }
    }

    #[test]
    fn multi_char_symbol() {
        assert_eq!(kinds(lex("lambda")), vec![sym("lambda")]);
    }

    #[test]
    fn symbol_with_hyphen() {
        assert_eq!(kinds(lex("my-var")), vec![sym("my-var")]);
    }

    #[test]
    fn symbol_with_question_mark() {
        assert_eq!(kinds(lex("nil?")), vec![sym("nil?")]);
    }

    // --- whacky whitespace ---

    #[test]
    fn tab_between_tokens() {
        assert_eq!(
            kinds(lex("(+\t1\t2)")),
            vec![
                TokenKind::LeftParen,
                sym("+"),
                sym("1"),
                sym("2"),
                TokenKind::RightParen
            ]
        );
    }

    #[test]
    fn carriage_return_between_tokens() {
        assert_eq!(
            kinds(lex("(+\r\n1\r\n2)")),
            vec![
                TokenKind::LeftParen,
                sym("+"),
                sym("1"),
                sym("2"),
                TokenKind::RightParen
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
            kinds(lex("(+  \t  1)")),
            vec![
                TokenKind::LeftParen,
                sym("+"),
                sym("1"),
                TokenKind::RightParen
            ]
        );
    }

    // --- string literals ---

    #[test]
    fn string_literal_no_spaces() {
        assert_eq!(kinds(lex("\"hello\"")), vec![sym("\"hello\"")]);
    }

    #[test]
    fn string_literal_with_spaces() {
        assert_eq!(
            kinds(lex("(print \"hello world\")")),
            vec![
                TokenKind::LeftParen,
                sym("print"),
                sym("\"hello world\""),
                TokenKind::RightParen
            ]
        );
    }

    #[test]
    fn two_string_literals_with_spaces() {
        assert_eq!(
            kinds(lex("(print \"hello world\" \"hello world\")")),
            vec![
                TokenKind::LeftParen,
                sym("print"),
                sym("\"hello world\""),
                sym("\"hello world\""),
                TokenKind::RightParen,
            ]
        );
    }

    #[test]
    fn empty_string_literal() {
        assert_eq!(kinds(lex("\"\"")), vec![sym("\"\"")]);
    }

    #[test]
    fn string_containing_parens() {
        assert_eq!(
            kinds(lex("\"(not a paren)\"")),
            vec![sym("\"(not a paren)\"")]
        );
    }

    // --- quote ---

    #[test]
    fn quote_atom() {
        assert_eq!(kinds(lex("'x")), vec![TokenKind::Quote, sym("x")]);
    }

    #[test]
    fn quote_number() {
        assert_eq!(kinds(lex("'42")), vec![TokenKind::Quote, sym("42")]);
    }

    #[test]
    fn quote_list() {
        assert_eq!(
            kinds(lex("'(+ 1 2)")),
            vec![
                TokenKind::Quote,
                TokenKind::LeftParen,
                sym("+"),
                sym("1"),
                sym("2"),
                TokenKind::RightParen
            ]
        );
    }

    #[test]
    fn quote_inside_expression() {
        assert_eq!(
            kinds(lex("(eq 'a 'b)")),
            vec![
                TokenKind::LeftParen,
                sym("eq"),
                TokenKind::Quote,
                sym("a"),
                TokenKind::Quote,
                sym("b"),
                TokenKind::RightParen,
            ]
        );
    }

    #[test]
    fn double_quote_shorthand() {
        assert_eq!(
            kinds(lex("''x")),
            vec![TokenKind::Quote, TokenKind::Quote, sym("x")]
        );
    }

    #[test]
    fn quote_string() {
        assert_eq!(
            kinds(lex("'\"hello\"")),
            vec![TokenKind::Quote, sym("\"hello\"")]
        );
    }

    // --- escape sequences ---

    #[test]
    fn escaped_quote_in_string() {
        assert_eq!(kinds(lex(r#""say \"hi\"""#)), vec![sym(r#""say \"hi\"""#)]);
    }

    #[test]
    fn escaped_backslash_in_string() {
        assert_eq!(kinds(lex(r#""foo\\bar""#)), vec![sym(r#""foo\\bar""#)]);
    }

    #[test]
    fn escaped_quote_at_end_of_string() {
        assert_eq!(kinds(lex(r#""hello\\""#)), vec![sym(r#""hello\\""#)]);
    }

    #[test]
    fn string_with_only_escaped_quote() {
        assert_eq!(kinds(lex(r#""\"""#)), vec![sym(r#""\"""#)]);
    }

    #[test]
    fn escaped_quote_does_not_break_surrounding_tokens() {
        assert_eq!(
            kinds(lex(r#"(print "say \"hi\"")"#)),
            vec![
                TokenKind::LeftParen,
                sym("print"),
                sym(r#""say \"hi\"""#),
                TokenKind::RightParen
            ]
        );
    }

    // --- backslash outside strings ---

    #[test]
    fn backslash_before_quote_shorthand() {
        assert_eq!(
            kinds(lex(r"\'x")),
            vec![sym(r"\"), TokenKind::Quote, sym("x")]
        );
    }

    #[test]
    fn backslash_before_left_paren() {
        assert_eq!(
            kinds(lex(r"\(foo)")),
            vec![
                sym(r"\"),
                TokenKind::LeftParen,
                sym("foo"),
                TokenKind::RightParen
            ]
        );
    }

    #[test]
    fn backslash_as_symbol_character() {
        assert_eq!(
            kinds(lex(r"(foo \ bar)")),
            vec![
                TokenKind::LeftParen,
                sym("foo"),
                sym(r"\"),
                sym("bar"),
                TokenKind::RightParen
            ]
        );
    }

    // --- multiple top-level forms ---

    #[test]
    fn two_top_level_expressions() {
        assert_eq!(
            kinds(lex("(+ 1 2) (- 3 4)")),
            vec![
                TokenKind::LeftParen,
                sym("+"),
                sym("1"),
                sym("2"),
                TokenKind::RightParen,
                TokenKind::LeftParen,
                sym("-"),
                sym("3"),
                sym("4"),
                TokenKind::RightParen,
            ]
        );
    }

    // --- spans ---

    #[test]
    fn symbol_span() {
        assert_eq!(lex("foo")[0], tok(sym("foo"), 0, 1));
    }

    #[test]
    fn symbol_span_after_whitespace() {
        // "   foo" — column increments per char, foo starts after 3 spaces
        assert_eq!(lex("   foo")[0], tok(sym("foo"), 0, 4));
    }

    #[test]
    fn paren_span() {
        assert_eq!(lex("(foo)")[0], tok(TokenKind::LeftParen, 0, 1));
    }

    #[test]
    fn second_line_span() {
        let tokens = lex("foo\nbar");
        assert_eq!(tokens[1], tok(sym("bar"), 1, 1));
    }

    #[test]
    fn column_resets_after_newline() {
        // "(+\n1)" — 1 is at line 1, column 1
        let tokens = lex("(+\n1)");
        assert_eq!(tokens[2], tok(sym("1"), 1, 1));
    }
}

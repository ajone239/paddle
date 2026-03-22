use thiserror::Error;

use crate::lexer::{Span, Token, TokenKind};

#[derive(Debug, PartialEq, Error)]
pub enum ParseError {
    #[error("No tokens were provided.")]
    EmptyInput,
    #[error("[{span}] Unexpected end of input.")]
    UnexpectedEof { span: Span },
    #[error("[{span}] Unexpected token.")]
    UnexpectedToken { span: Span },
}

#[derive(Debug, PartialEq)]
pub enum Expr<'a> {
    Atom(&'a str, Span),
    List(Vec<Expr<'a>>, Span),
}

type Exprest<'a> = (Expr<'a>, &'a [Token<'a>]);
type ParseResult<'a> = Result<Exprest<'a>, ParseError>;

pub fn parse_expr<'a>(tokens: &'a [Token<'a>]) -> ParseResult<'a> {
    if tokens.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    if let [
        Token {
            kind: TokenKind::Symbol(s),
            span,
        },
        rest @ ..,
    ] = tokens
    {
        return Ok((Expr::Atom(s, *span), rest));
    }

    let first_token = &tokens[0];
    match first_token.kind {
        TokenKind::LeftParen => parse_list(&tokens[1..]),
        TokenKind::Quote => {
            if tokens.len() < 2 {
                return Err(ParseError::UnexpectedEof {
                    span: first_token.span,
                });
            }
            let (expr, rest) = parse_expr(&tokens[1..])?;
            Ok((
                Expr::List(
                    vec![Expr::Atom("quote", first_token.span), expr],
                    first_token.span,
                ),
                rest,
            ))
        }
        _ => Err(ParseError::UnexpectedToken {
            span: first_token.span,
        }),
    }
}

fn parse_list<'a>(tokens: &'a [Token<'a>]) -> ParseResult<'a> {
    let mut list = vec![];
    let mut i = 0;

    let span = tokens[0].span;

    loop {
        if i >= tokens.len() {
            return Err(ParseError::UnexpectedEof {
                span: tokens[tokens.len() - 1].span,
            });
        }

        let token = &tokens[i];

        if matches!(token.kind, TokenKind::RightParen) {
            break;
        }

        let (expr, rest) = parse_expr(&tokens[i..])?;

        i = tokens.len() - rest.len();
        list.push(expr);
    }

    let rest = if i >= tokens.len() {
        &[]
    } else {
        &tokens[i + 1..]
    };

    return Ok((Expr::List(list, span), rest));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    macro_rules! sp {
        ($line:expr, $col:expr) => {
            Span {
                line: $line,
                column: $col,
            }
        };
    }

    fn atom(s: &str, line: usize, column: usize) -> Expr<'_> {
        Expr::Atom(s, Span { line, column })
    }

    fn strip_spans(expr: Expr) -> Expr {
        match expr {
            Expr::Atom(text, _) => Expr::Atom(text, sp!(0, 0)),
            Expr::List(items, _) => {
                Expr::List(items.into_iter().map(strip_spans).collect(), sp!(0, 0))
            }
        }
    }

    fn a(s: &str) -> Expr<'_> {
        Expr::Atom(s, sp!(0, 0))
    }

    fn l(items: Vec<Expr>) -> Expr {
        Expr::List(items, sp!(0, 0))
    }

    // --- structure (spans ignored) ---

    #[test]
    fn single_atom() {
        let tokens = lex("foo");
        let (expr, rest) = parse_expr(&tokens).unwrap();
        assert_eq!(strip_spans(expr), a("foo"));
        assert!(rest.is_empty());
    }

    #[test]
    fn double_atom() {
        let tokens = lex("foo bar");
        let (expr, rest) = parse_expr(&tokens).unwrap();
        assert_eq!(strip_spans(expr), a("foo"));
        assert!(!rest.is_empty());
    }

    #[test]
    fn empty_list() {
        let tokens = lex("()");
        let (expr, rest) = parse_expr(&tokens).unwrap();
        assert_eq!(strip_spans(expr), l(vec![]));
        assert!(rest.is_empty());
    }

    #[test]
    fn simple_list() {
        let tokens = lex("(+ 1 2)");
        let (expr, rest) = parse_expr(&tokens).unwrap();
        assert_eq!(strip_spans(expr), l(vec![a("+"), a("1"), a("2")]));
        assert!(rest.is_empty());
    }

    #[test]
    fn nested_list() {
        let tokens = lex("(+ (- 3 1) 2)");
        let (expr, rest) = parse_expr(&tokens).unwrap();
        assert_eq!(
            strip_spans(expr),
            l(vec![a("+"), l(vec![a("-"), a("3"), a("1")]), a("2")])
        );
        assert!(rest.is_empty());
    }

    #[test]
    fn deeply_nested() {
        let tokens = lex("(a (b (c)))");
        let (expr, rest) = parse_expr(&tokens).unwrap();
        assert_eq!(
            strip_spans(expr),
            l(vec![a("a"), l(vec![a("b"), l(vec![a("c")])])])
        );
        assert!(rest.is_empty());
    }

    #[test]
    fn quote_inside_list() {
        let tokens = lex("(foo 'x)");
        let (expr, rest) = parse_expr(&tokens).unwrap();
        assert_eq!(
            strip_spans(expr),
            l(vec![a("foo"), l(vec![a("quote"), a("x")])])
        );
        assert!(rest.is_empty());
    }

    #[test]
    fn quote_atom() {
        let tokens = lex("'x");
        let (expr, rest) = parse_expr(&tokens).unwrap();
        assert_eq!(strip_spans(expr), l(vec![a("quote"), a("x")]));
        assert!(rest.is_empty());
    }

    #[test]
    fn double_quote() {
        let tokens = lex("''x");
        let (expr, rest) = parse_expr(&tokens).unwrap();
        assert_eq!(
            strip_spans(expr),
            l(vec![a("quote"), l(vec![a("quote"), a("x")])])
        );
        assert!(rest.is_empty());
    }

    #[test]
    fn quote_list() {
        let tokens = lex("'(+ 1 2)");
        let (expr, rest) = parse_expr(&tokens).unwrap();
        assert_eq!(
            strip_spans(expr),
            l(vec![a("quote"), l(vec![a("+"), a("1"), a("2")])])
        );
        assert!(rest.is_empty());
    }

    // --- spans ---

    #[test]
    fn atom_span() {
        let tokens = lex("foo");
        let (expr, _) = parse_expr(&tokens).unwrap();
        assert_eq!(expr, atom("foo", 0, 1));
    }

    #[test]
    fn atom_span_after_whitespace() {
        // "   foo" — foo starts at column 4
        let tokens = lex("   foo");
        let (expr, _) = parse_expr(&tokens).unwrap();
        assert_eq!(expr, atom("foo", 0, 4));
    }

    #[test]
    fn atom_on_second_line() {
        let tokens = lex("foo\nbar");
        let (_, rest) = parse_expr(&tokens).unwrap();
        let (expr, _) = parse_expr(rest).unwrap();
        assert_eq!(expr, atom("bar", 1, 1));
    }

    #[test]
    fn list_span_is_first_element() {
        // span of a list is the first token inside it (the opening paren is consumed)
        let tokens = lex("(+ 1 2)");
        let (expr, _) = parse_expr(&tokens).unwrap();
        if let Expr::List(_, span) = expr {
            assert_eq!(span, sp!(0, 2)); // '+' is at column 2
        } else {
            panic!("expected list");
        }
    }

    // --- errors ---

    #[test]
    fn empty_input() {
        let tokens = lex("");
        assert_eq!(parse_expr(&tokens), Err(ParseError::EmptyInput));
    }

    #[test]
    fn unclosed_paren() {
        let tokens = lex("(+ 1 2");
        assert!(matches!(
            parse_expr(&tokens),
            Err(ParseError::UnexpectedEof { .. })
        ));
    }

    #[test]
    fn bare_quote_at_end() {
        let tokens = lex("'");
        assert!(matches!(
            parse_expr(&tokens),
            Err(ParseError::UnexpectedEof { .. })
        ));
    }

    #[test]
    fn unexpected_close_paren() {
        let tokens = lex(")");
        assert!(matches!(
            parse_expr(&tokens),
            Err(ParseError::UnexpectedToken { .. })
        ));
    }
}

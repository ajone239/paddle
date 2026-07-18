use std::rc::Rc;

use crate::eval::value::{Form, Value};
use crate::lexer::Span;
use crate::parser::Expr;

pub fn lower(ast: &Expr) -> Value {
    quote_eval(ast)
}

fn quote_eval(ast: &Expr) -> Value {
    match ast {
        Expr::Atom(atom, span) => classify(atom, span.clone()),
        Expr::List(list, span) => {
            // TODO(austin.jones): I don't think this is right way to handle the
            //                     list span
            let mut rv = Value::Nil(span.clone());

            for val in list.iter().map(quote_eval).rev() {
                let span = val.get_span();
                rv = Value::Cons(Rc::new((val, rv)), span);
            }
            rv
        }
    }
}

fn classify(atom: &str, span: Span) -> Value {
    if let Ok(num) = atom.parse::<f64>() {
        return Value::Num(num, span);
    }

    if let Some(form) = Form::try_parse(atom) {
        return Value::Form(form, span);
    }

    match atom {
        "nil" => Value::Nil(span),
        "#t" => Value::Bool(true, span),
        "#f" => Value::Bool(false, span),
        _ if atom.starts_with('"') && atom.ends_with('"') => Value::Str(
            Rc::from(atom.strip_prefix("\"").unwrap().strip_suffix("\"").unwrap()),
            span,
        ),
        _ => Value::Symbol(Rc::from(atom), span),
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::lower;
    use crate::eval::tests::strip_value_span;
    use crate::eval::value::Value;
    use crate::lexer::{Span, lex};
    use crate::parser::parse_expr;

    fn lower_str(s: &str) -> Value {
        let tokens = lex(s);
        let (ast, _) = parse_expr(&tokens).unwrap();
        strip_value_span(&lower(&ast))
    }

    fn cons(head: Value, tail: Value) -> Value {
        Value::Cons(Rc::new((head, tail)), Span::default())
    }

    fn num(n: f64) -> Value {
        Value::Num(n, Span::default())
    }

    fn sym(s: &str) -> Value {
        Value::Symbol(Rc::from(s), Span::default())
    }

    #[test]
    fn list_three_elements() {
        assert_eq!(
            lower_str("(1 2 3)"),
            cons(
                num(1.0),
                cons(num(2.0), cons(num(3.0), Value::Nil(Span::default())))
            )
        );
    }

    #[test]
    fn empty_list() {
        // empty list lowers to a single Cons with Nil head and Nil tail
        assert_eq!(lower_str("()"), Value::Nil(Span::default()));
    }

    #[test]
    fn nested_lists() {
        assert_eq!(
            lower_str("((1 1) (2 2) (3 3))"),
            cons(
                cons(num(1.0), cons(num(1.0), Value::Nil(Span::default()))),
                cons(
                    cons(num(2.0), cons(num(2.0), Value::Nil(Span::default()))),
                    cons(
                        cons(num(3.0), cons(num(3.0), Value::Nil(Span::default()))),
                        Value::Nil(Span::default())
                    )
                )
            )
        );
    }

    #[test]
    fn single_element_list() {
        assert_eq!(
            lower_str("(42)"),
            cons(num(42.0), Value::Nil(Span::default()))
        );
    }

    #[test]
    fn atom_num() {
        assert_eq!(lower_str("7"), num(7.0));
    }

    #[test]
    fn atom_symbol() {
        assert_eq!(lower_str("foo"), sym("foo"));
    }

    #[test]
    fn mixed_types() {
        assert_eq!(
            lower_str("(1 foo #t)"),
            cons(
                num(1.0),
                cons(
                    sym("foo"),
                    cons(
                        Value::Bool(true, Span::default()),
                        Value::Nil(Span::default())
                    )
                )
            )
        );
    }

    #[test]
    fn deeply_nested() {
        assert_eq!(
            lower_str("((()))"),
            cons(
                cons(Value::Nil(Span::default()), Value::Nil(Span::default())),
                Value::Nil(Span::default())
            )
        );
    }
}

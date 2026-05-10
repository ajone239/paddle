use std::rc::Rc;

use crate::eval::value::{Form, Value};
use crate::parser::Expr;

pub fn lower(ast: &Expr) -> Value {
    quote_eval(ast)
}

fn quote_eval(ast: &Expr) -> Value {
    match ast {
        Expr::Atom(atom, _) => classify(atom),
        Expr::List(list, _) => {
            let mut rv = Value::Nil;
            for val in list.iter().map(quote_eval).rev() {
                rv = Value::Cons(Rc::new((val, rv)));
            }
            rv
        }
    }
}

fn classify(atom: &str) -> Value {
    if let Ok(num) = atom.parse::<f64>() {
        return Value::Num(num);
    }

    if let Some(form) = Form::try_parse(atom) {
        return Value::Form(form);
    }

    match atom {
        "nil" => Value::Nil,
        "#t" => Value::Bool(true),
        "#f" => Value::Bool(false),
        _ if atom.starts_with('"') && atom.ends_with('"') => Value::Str(
            atom.strip_prefix("\"")
                .unwrap()
                .strip_suffix("\"")
                .unwrap()
                .to_owned(),
        ),
        _ => Value::Symbol(atom.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::lower;
    use crate::eval::value::Value;
    use crate::lexer::lex;
    use crate::parser::parse_expr;

    fn lower_str(s: &str) -> Value {
        let tokens = lex(s);
        let (ast, _) = parse_expr(&tokens).unwrap();
        lower(&ast)
    }

    fn cons(head: Value, tail: Value) -> Value {
        Value::Cons(Rc::new((head, tail)))
    }

    fn num(n: f64) -> Value {
        Value::Num(n)
    }

    fn sym(s: &str) -> Value {
        Value::Symbol(s.to_owned())
    }

    #[test]
    fn list_three_elements() {
        assert_eq!(
            lower_str("(1 2 3)"),
            cons(num(1.0), cons(num(2.0), cons(num(3.0), Value::Nil)))
        );
    }

    #[test]
    fn empty_list() {
        // empty list lowers to a single Cons with Nil head and Nil tail
        assert_eq!(lower_str("()"), Value::Nil);
    }

    #[test]
    fn nested_lists() {
        assert_eq!(
            lower_str("((1 1) (2 2) (3 3))"),
            cons(
                cons(num(1.0), cons(num(1.0), Value::Nil)),
                cons(
                    cons(num(2.0), cons(num(2.0), Value::Nil)),
                    cons(cons(num(3.0), cons(num(3.0), Value::Nil)), Value::Nil)
                )
            )
        );
    }

    #[test]
    fn single_element_list() {
        assert_eq!(lower_str("(42)"), cons(num(42.0), Value::Nil));
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
                cons(sym("foo"), cons(Value::Bool(true), Value::Nil))
            )
        );
    }

    #[test]
    fn deeply_nested() {
        assert_eq!(
            lower_str("((()))"),
            cons(cons(Value::Nil, Value::Nil), Value::Nil)
        );
    }
}

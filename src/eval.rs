use core::panic;

use crate::parser::Expr;

#[derive(Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Num(f64),
    Symbol(String),
    Form(Form),
    Str(String),
    // TODO(ajone239): move this to a ref when copies get expensive
    List(Vec<Value>),
}

#[derive(Debug, PartialEq)]
pub enum Form {
    Quote,
}

impl Form {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "quote" | "'" => Some(Self::Quote),
            _ => None,
        }
    }
}

pub fn eval<'a>(ast: &Expr<'a>) -> Value {
    match ast {
        Expr::Atom(atom, _) => resolve(atom),
        Expr::List(list, _) => apply(&list),
    }
}

fn quote_eval<'a>(ast: &Expr<'a>) -> Value {
    match ast {
        Expr::Atom(atom, _) => resolve(atom),
        Expr::List(list, _) => Value::List(list.iter().map(quote_eval).collect()),
    }
}

fn resolve<'a>(atom: &'a str) -> Value {
    if let Ok(num) = atom.parse::<f64>() {
        return Value::Num(num);
    }

    match atom {
        "nil" => Value::Nil,
        "#t" => Value::Bool(true),
        "#f" => Value::Bool(false),
        s if Form::from_str(s).is_some() => Value::Form(Form::from_str(s).unwrap()),
        _ if atom.chars().nth(0).unwrap() == '"' => Value::Str(atom.to_owned()),
        _ => Value::Symbol(atom.to_owned()),
    }
}

fn apply<'a>(list: &[Expr<'a>]) -> Value {
    if list.is_empty() {
        return Value::Nil;
    }

    let mut vals = list.iter().map(|e| eval(e));

    match vals.next().unwrap() {
        Value::Form(Form::Quote) => {
            let tail = &list[1..];
            return match tail.len() {
                1 => quote_eval(&list[1]),
                _ => Value::List(tail.iter().map(quote_eval).collect()),
            };
        }
        Value::Symbol(func) => call(&func, &vals.collect::<Vec<Value>>()),
        _ => panic!("shouldn't hit this"),
    }
}

fn call<'a>(func: &str, args: &[Value]) -> Value {
    let mut args = args.iter().map(|v| match v {
        Value::Num(n) => n,
        _ => todo!(),
    });

    let num = match func {
        "+" => args.fold(0.0, |acc, x| acc + x),
        "*" => args.fold(1.0, |acc, x| acc * x),
        "-" => {
            let init = *args.next().expect("must have args with -");
            args.fold(init, |acc, x| acc - x)
        }
        "/" => {
            let init = *args.next().expect("must have args with /");
            args.fold(init, |acc, x| acc / x)
        }
        _ => panic!("operation not supported"),
    };
    Value::Num(num)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse_expr;

    fn eval_str(s: &str) -> Value {
        let tokens = lex(s);
        let (expr, _) = parse_expr(&tokens).unwrap();
        eval(&expr)
    }

    fn num(n: f64) -> Value {
        Value::Num(n)
    }

    // --- atoms ---

    #[test]
    fn integer_literal() {
        assert_eq!(eval_str("42"), num(42.0));
    }

    #[test]
    fn float_literal() {
        assert_eq!(eval_str("3.14"), num(3.14));
    }

    #[test]
    fn negative_literal() {
        assert_eq!(eval_str("-7"), num(-7.0));
    }

    #[test]
    fn nil_literal() {
        assert_eq!(eval_str("nil"), Value::Nil);
    }

    #[test]
    fn true_literal() {
        assert_eq!(eval_str("#t"), Value::Bool(true));
    }

    #[test]
    fn false_literal() {
        assert_eq!(eval_str("#f"), Value::Bool(false));
    }

    // --- addition ---

    #[test]
    fn add_two() {
        assert_eq!(eval_str("(+ 1 2)"), num(3.0));
    }

    #[test]
    fn add_three() {
        assert_eq!(eval_str("(+ 1 2 3)"), num(6.0));
    }

    #[test]
    fn add_no_args() {
        assert_eq!(eval_str("(+)"), num(0.0));
    }

    #[test]
    fn add_one_arg() {
        assert_eq!(eval_str("(+ 5)"), num(5.0));
    }

    // --- subtraction ---

    #[test]
    fn sub_two() {
        assert_eq!(eval_str("(- 10 3)"), num(7.0));
    }

    #[test]
    fn sub_three() {
        assert_eq!(eval_str("(- 10 3 2)"), num(5.0));
    }

    #[test]
    fn sub_one_arg() {
        assert_eq!(eval_str("(- 5)"), num(5.0));
    }

    // --- multiplication ---

    #[test]
    fn mul_two() {
        assert_eq!(eval_str("(* 3 4)"), num(12.0));
    }

    #[test]
    fn mul_no_args() {
        assert_eq!(eval_str("(*)"), num(1.0));
    }

    #[test]
    fn mul_one_arg() {
        assert_eq!(eval_str("(* 7)"), num(7.0));
    }

    // --- division ---

    #[test]
    fn div_two() {
        assert_eq!(eval_str("(/ 10 2)"), num(5.0));
    }

    #[test]
    fn div_three() {
        assert_eq!(eval_str("(/ 24 4 3)"), num(2.0));
    }

    // --- nesting ---

    #[test]
    fn nested_add() {
        assert_eq!(eval_str("(+ 1 (+ 2 3))"), num(6.0));
    }

    #[test]
    fn nested_mixed() {
        assert_eq!(eval_str("(* (+ 1 2) (- 5 2))"), num(9.0));
    }

    #[test]
    fn deeply_nested() {
        assert_eq!(eval_str("(+ 1 (* 2 (- 10 (/ 6 2))))"), num(15.0));
    }

    // --- empty list ---

    #[test]
    fn empty_list() {
        assert_eq!(eval_str("()"), Value::Nil);
    }
}

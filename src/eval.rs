use core::panic;

use crate::parser::Expr;

#[derive(Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Num(f64),
    Symbol(String),
    Str(String),
    List(Vec<Value>),
}

pub fn eval<'a>(ast: &Expr<'a>) -> Value {
    match ast {
        Expr::Atom(atom, _) => resolve(atom),
        Expr::List(list, _) => apply(&list),
    }
}

fn apply<'a>(list: &[Expr<'a>]) -> Value {
    if list.is_empty() {
        return Value::Nil;
    }

    let vals: Vec<_> = list.iter().map(|e| eval(e)).collect();
    let func = &vals[0];
    let args = &vals[1..];

    call(func, args)
}

fn call<'a>(func: &Value, args: &[Value]) -> Value {
    let mut args = args.iter().map(|v| match v {
        Value::Num(n) => n,
        _ => todo!(),
    });

    let Value::Symbol(s) = func else {
        panic!("bad expr head")
    };

    let num = match s.as_str() {
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

fn resolve<'a>(atom: &'a str) -> Value {
    if let Ok(num) = atom.parse::<f64>() {
        return Value::Num(num);
    }

    match atom {
        "nil" => Value::Nil,
        "#t" => Value::Bool(true),
        "#f" => Value::Bool(false),
        _ if atom.chars().nth(0).unwrap() == '"' => Value::Str(atom.to_owned()),
        _ => Value::Symbol(atom.to_owned()),
    }
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

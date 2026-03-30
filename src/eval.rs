use core::panic;
use std::{collections::HashMap, rc::Rc};

use crate::parser::Expr;

#[derive(Debug)]
pub struct Env {
    env: HashMap<String, Value>,
}

impl Env {
    pub fn default() -> Self {
        let env = HashMap::new();
        Self { env }
    }

    fn define(&mut self, name: &str, value: Value) {
        self.env.insert(name.to_owned(), value);
    }

    fn resolve(&self, name: &str) -> Option<&Value> {
        self.env.get(name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Num(f64),
    Symbol(String),
    Form(Form),
    Str(String),
    // TODO(ajone239): move this to a ref when copies get expensive
    List(Vec<Value>),
    Func {
        name: String,
        args: Vec<String>,
        body: Rc<Value>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Form {
    Quote,
    Define,
}

impl Form {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "quote" | "'" => Some(Self::Quote),
            "define" | "def" => Some(Self::Define),
            _ => None,
        }
    }
}

pub fn eval(ast: &Expr, env: &mut Env) -> Value {
    match ast {
        Expr::Atom(atom, _) => resolve(atom, env),
        Expr::List(list, _) => apply(&list, env),
    }
}

fn quote_eval_list(ast: &[Expr]) -> Value {
    match ast.len() {
        1 => quote_eval(&ast[0]),
        _ => {
            let list = ast.iter().map(|expr| quote_eval(expr)).collect();
            Value::List(list)
        }
    }
}

fn quote_eval(ast: &Expr) -> Value {
    match ast {
        Expr::Atom(atom, _) => classify(atom),
        Expr::List(list, _) => {
            let list = list.iter().map(quote_eval).collect();
            Value::List(list)
        }
    }
}

fn resolve(atom: &str, env: &Env) -> Value {
    if let Some(form) = Form::from_str(atom) {
        return Value::Form(form);
    }

    if let Some(val) = env.resolve(atom) {
        return val.clone();
    }

    classify(atom)
}

fn classify(atom: &str) -> Value {
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

fn apply(list: &[Expr], env: &mut Env) -> Value {
    if list.is_empty() {
        return Value::Nil;
    }

    // env captured here -- very fragile
    let mut vals = list.iter().map(|e| eval(e, env));

    match vals.next().unwrap() {
        Value::Form(Form::Quote) => {
            let tail = &list[1..];
            return quote_eval_list(tail);
        }
        Value::Form(Form::Define) => {
            if list.len() < 3 {
                panic!("Bad define");
            }
            let head = &list[1];
            let tail = &list[2..];

            match head {
                Expr::Atom(atom, _) => {
                    let value = eval(&tail[0], env);
                    env.define(atom, value);
                    Value::Nil
                }
                Expr::List(exprs, _) => {
                    let mut atoms =
                        exprs
                            .iter()
                            .filter(|e| matches!(e, Expr::Atom(_, _)))
                            .map(|e| match e {
                                Expr::Atom(a, _) => (*a).to_owned(),
                                Expr::List(_, _) => panic!("come on man"),
                            });
                    let name = atoms.next().unwrap();
                    let body = quote_eval_list(tail);

                    let func = Value::Func {
                        name: name.clone(),
                        args: atoms.collect(),
                        body: Rc::new(body),
                    };

                    env.define(&name, func);

                    Value::Nil
                }
            }
        }
        // env used here
        Value::Symbol(func) => call(&func, &vals.collect::<Vec<Value>>()),
        Value::Func {
            name: _,
            args,
            body,
        } => {
            // define the args in context
            // TODO(ajone239): cache old context
            let vals: Vec<Value> = vals.collect();

            if vals.len() != args.len() {
                panic!("Expected {} args got {}", args.len(), vals.len());
            }

            for (arg, val) in args.iter().zip(vals) {
                env.define(arg, val);
            }

            // eval the body with the new env

            // return the value
            todo!()
        }
        _ => panic!("shouldn't hit this"),
    }
}

fn call(func: &str, args: &[Value]) -> Value {
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
        let mut env = Env::default();
        let tokens = lex(s);
        let (expr, _) = parse_expr(&tokens).unwrap();
        eval(&expr, &mut env)
    }

    fn eval_str_env(exprs: &[&str]) -> Value {
        let mut env = Env::default();

        let mut last = None;

        for expr in exprs {
            let tokens = lex(expr);
            let (e, _) = parse_expr(&tokens).unwrap();
            let val = eval(&e, &mut env);
            last = Some(val);
        }

        last.unwrap()
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

    // --- quote ---

    #[test]
    fn quote_symbol() {
        assert_eq!(eval_str("(quote x)"), Value::Symbol("x".to_owned()));
    }

    #[test]
    fn quote_number() {
        assert_eq!(eval_str("(quote 42)"), num(42.0));
        assert_eq!(eval_str("'42"), num(42.0));
    }

    #[test]
    fn quote_nil() {
        assert_eq!(eval_str("(quote nil)"), Value::Nil);
    }

    #[test]
    fn quote_define() {
        assert_eq!(
            eval_str("(quote define)"),
            Value::Symbol("define".to_owned())
        );
    }

    #[test]
    fn quote_list() {
        assert_eq!(
            eval_str("(quote (1 2 3))"),
            Value::List(vec![num(1.0), num(2.0), num(3.0)])
        );
    }

    #[test]
    fn quote_suppresses_eval() {
        assert_eq!(
            eval_str("(quote (+ 1 2))"),
            Value::List(vec![Value::Symbol("+".to_owned()), num(1.0), num(2.0),])
        );
    }

    #[test]
    fn define_and_resolve() {
        assert_eq!(eval_str_env(&vec!["(def x 5)", "(+ x 1)"]), num(6.0));
    }

    #[test]
    fn redefine() {
        assert_eq!(eval_str_env(&vec!["(def x 1)", "(def x 2)", "x"]), num(2.0));
    }

    #[test]
    fn define_returns_nil() {
        assert_eq!(eval_str("(def x 5)"), Value::Nil);
    }

    #[test]
    fn define_expression_value() {
        assert_eq!(eval_str_env(&vec!["(def x (+ 1 2))", "x"]), num(3.0));
    }

    #[test]
    fn undefined_symbol() {
        assert_eq!(eval_str("x"), Value::Symbol("x".to_owned()));
    }
}

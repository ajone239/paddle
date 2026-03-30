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

pub fn lower(ast: &Expr) -> Value {
    quote_eval(ast)
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

fn classify(atom: &str) -> Value {
    if let Ok(num) = atom.parse::<f64>() {
        return Value::Num(num);
    }

    if let Some(form) = Form::from_str(atom) {
        return Value::Form(form);
    }

    match atom {
        "nil" => Value::Nil,
        "#t" => Value::Bool(true),
        "#f" => Value::Bool(false),
        _ if atom.chars().nth(0).unwrap() == '"' => Value::Str(atom.to_owned()),
        _ => Value::Symbol(atom.to_owned()),
    }
}

pub fn eval(ast: &Value, env: &mut Env) -> Value {
    match ast {
        Value::Symbol(atom) => resolve(&atom, env),
        // copy here plz fix
        Value::List(list) => apply(list.clone(), env),
        _ => ast.clone(),
    }
}

fn resolve(atom: &str, env: &Env) -> Value {
    if let Some(val) = env.resolve(atom) {
        return val.clone();
    }

    classify(atom)
}

fn apply(list: Vec<Value>, env: &mut Env) -> Value {
    if list.is_empty() {
        return Value::Nil;
    }

    match &list[0] {
        Value::Form(Form::Quote) => {
            let tail = &list[1..];
            match tail.len() {
                1 => tail[0].clone(),
                _ => Value::List(tail.iter().map(|v| v.clone()).collect()),
            }
        }
        Value::Form(Form::Define) => {
            if list.len() < 3 {
                panic!("Bad define");
            }
            let head = &list[1];
            let tail = &list[2..];

            match head {
                Value::Symbol(atom) => {
                    let value = eval(&tail[0], env);
                    env.define(atom, value);
                    Value::Nil
                }
                Value::List(exprs) => {
                    let mut atoms = exprs.iter().map(|e| match e {
                        Value::Symbol(a) => (*a).to_owned(),
                        _ => panic!("come on man"),
                    });
                    let name = atoms.next().unwrap();
                    let body = Value::List(tail.to_vec());

                    let func = Value::Func {
                        name: name.clone(),
                        args: atoms.collect(),
                        body: Rc::new(body),
                    };

                    env.define(&name, func);

                    Value::Nil
                }
                _ => panic!("bad define"),
            }
        }
        // env used here
        Value::Symbol(func) => {
            let args: Vec<Value> = list[1..].iter().map(|v| eval(v, env)).collect();
            call(&func, &args)
        }
        // this almost never gets hit because of eval order
        Value::Func {
            name: _,
            args,
            body,
        } => {
            // define the args in context
            // TODO(ajone239): cache old context
            let vals = &list[1..];

            if vals.len() != args.len() {
                panic!("Expected {} args got {}", args.len(), vals.len());
            }

            for (arg, val) in args.iter().zip(vals) {
                env.define(arg, val.clone());
            }

            // eval the body with the new env
            // return the value
            eval(body, env)
        }
        _ => panic!("shouldn't hit this"),
    }
}

fn call(func: &str, args: &[Value]) -> Value {
    let mut args = args.iter().map(|v| match v {
        Value::Num(n) => n,
        _ => todo!("bad call args: {:?}", args),
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
        _ => panic!("operation not supported: {:?}", func),
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
        let expr = lower(&expr);
        eval(&expr, &mut env)
    }

    fn eval_str_env(exprs: &[&str]) -> Value {
        let mut env = Env::default();

        let mut last = None;

        for expr in exprs {
            let tokens = lex(expr);
            let (e, _) = parse_expr(&tokens).unwrap();
            let e = lower(&e);
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

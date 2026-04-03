use core::panic;
use std::cell::RefCell;
use std::{ops::Deref, rc::Rc};

use crate::env::Env;
use crate::parser::Expr;

pub type Builtin = fn(&[Value]) -> Value;

#[derive(Debug, Clone, Copy)]
pub struct BuiltinFn(pub Builtin);

impl PartialEq for BuiltinFn {
    fn eq(&self, _: &Self) -> bool {
        false
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
    Progn(Vec<Value>),
    Builtin(BuiltinFn),
    Func {
        name: String,
        args: Vec<String>,
        body: Rc<Value>,
    },
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(val) => *val,
            Value::Num(num) => num.ne(&0.0),
            Value::Str(s) => !s.is_empty(),
            Value::List(v) | Value::Progn(v) => !v.is_empty(),
            Value::Symbol(_)
            | Value::Form(_)
            | Value::Builtin(_)
            | Value::Func {
                name: _,
                args: _,
                body: _,
            } => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Form {
    If,
    Quote,
    Define,
}

impl Form {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "if" => Some(Self::If),
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
        _ if atom.starts_with('\"') => Value::Str(
            atom.strip_prefix("\"")
                .unwrap()
                .strip_suffix("\"")
                .unwrap()
                .to_owned(),
        ),
        _ => Value::Symbol(atom.to_owned()),
    }
}

pub fn eval(ast: &Value, env: Rc<RefCell<Env>>) -> Value {
    match ast {
        Value::Symbol(atom) => resolve(atom, env),
        Value::List(list) if list.is_empty() => Value::Nil,
        Value::List(list) => {
            let head = &list[0];

            match head {
                Value::Form(Form::Quote) => {
                    return list[1].clone();
                }
                Value::Form(Form::Define) => {
                    if list.len() < 3 {
                        panic!("Bad define");
                    }

                    define(&list[1], &list[2..], env);

                    return Value::Nil;
                }
                Value::Form(Form::If) => {
                    if list.len() < 3 {
                        panic!("Bad if");
                    }

                    let cond = &list[1];
                    let t_branch = &list[2];
                    let f_branch = &list[3];

                    let cond = eval(cond, env.clone());

                    return if cond.truthy() {
                        eval(t_branch, env)
                    } else {
                        eval(f_branch, env)
                    };
                }
                _ => {}
            }

            let list: Vec<Value> = list.iter().map(|v| eval(v, env.clone())).collect();
            apply(&list, env)
        }
        _ => ast.clone(),
    }
}

fn resolve(atom: &str, env: Rc<RefCell<Env>>) -> Value {
    match env.borrow().resolve(atom) {
        Some(val) => val,
        _ => panic!("symbol {} undefined", atom),
    }
}

fn apply(list: &[Value], env: Rc<RefCell<Env>>) -> Value {
    let args = &list[1..];

    match &list[0] {
        Value::Builtin(f) => f.0(args),
        Value::Func {
            name: _,
            args: fargs,
            body,
        } => {
            let env = Rc::new(RefCell::new(Env::new_child(env)));

            if fargs.len() != args.len() {
                panic!("Expected {} args got {}", fargs.len(), args.len());
            }

            for (arg, val) in fargs.iter().zip(args) {
                env.borrow_mut().define(arg, val.clone());
            }

            // eval the body with the new env
            // return the value
            match body.deref() {
                Value::Progn(body) => {
                    for b in &body[..body.len() - 1] {
                        eval(b, env.clone());
                    }
                    eval(body.last().unwrap(), env.clone())
                }
                _ => eval(body, env),
            }
        }
        v => v.clone(),
    }
}

fn define(head: &Value, tail: &[Value], env: Rc<RefCell<Env>>) {
    match head {
        Value::Symbol(atom) => {
            let value = eval(&tail[0], env.clone());
            env.borrow_mut().define(atom, value);
        }
        Value::List(exprs) => {
            let mut atoms = exprs.iter().map(|e| match e {
                Value::Symbol(a) => (*a).to_owned(),
                _ => panic!("come on man"),
            });
            let name = atoms.next().unwrap();
            let body = if tail.len() == 1 {
                Rc::new(tail[0].clone())
            } else {
                Rc::new(Value::Progn(tail.to_vec()))
            };

            let func = Value::Func {
                name: name.clone(),
                args: atoms.collect(),
                body,
            };

            env.borrow_mut().define(&name, func);
        }
        _ => panic!("bad define"),
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse_expr;

    fn eval_str(s: &str) -> Value {
        let env = Env::default();
        let tokens = lex(s);
        let (expr, _) = parse_expr(&tokens).unwrap();
        let expr = lower(&expr);
        eval(&expr, Rc::new(RefCell::new(env)))
    }

    fn eval_str_env(exprs: &[&str]) -> Value {
        let env = Rc::new(RefCell::new(Env::default()));

        let mut last = None;

        for expr in exprs {
            let tokens = lex(expr);
            let (e, _) = parse_expr(&tokens).unwrap();
            let e = lower(&e);
            let val = eval(&e, env.clone());
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

    // --- env vars ---

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
    #[should_panic]
    fn undefined_symbol() {
        assert_eq!(eval_str("x"), Value::Symbol("x".to_owned()));
    }

    // --- env funs ---

    #[test]
    fn define_func_and_call() {
        assert_eq!(
            eval_str_env(&["(def (double x) (* x 2))", "(double 3)"]),
            num(6.0)
        );
    }

    #[test]
    #[should_panic]
    fn define_func_scope() {
        assert_eq!(
            eval_str_env(&["(def (double x) (* x 2))", "(double 3)", "x"]),
            num(6.0)
        );
    }

    #[test]
    fn define_func_shadow() {
        assert_eq!(
            eval_str_env(&["(def x 10)", "(def (double x) (* x 2))", "(double 3)", "x"]),
            num(10.0)
        );
    }

    #[test]
    fn define_func_two_args() {
        assert_eq!(
            eval_str_env(&["(def (add x y) (+ x y))", "(add 3 4)"]),
            num(7.0)
        );
    }

    #[test]
    fn define_func_no_args() {
        assert_eq!(
            eval_str_env(&["(def (forty-two) 42)", "(forty-two)"]),
            num(42.0)
        );
    }

    #[test]
    fn define_func_multi_body() {
        assert_eq!(
            eval_str_env(&["(def (f x) (+ x 1) (* x 2))", "(f 3)"]),
            num(6.0)
        );
    }

    #[test]
    fn define_func_returns_nil() {
        assert_eq!(eval_str("(def (f x) (+ x 1))"), Value::Nil);
    }

    #[test]
    fn define_func_fact() {
        assert_eq!(
            eval_str_env(&vec![
                "(def (fact n) (if (< n 1) 1 (* n (fact (- n 1)))))",
                "(fact 5)"
            ]),
            Value::Num(120.0)
        );
    }

    #[test]
    fn define_func_fact_cute() {
        assert_eq!(
            eval_str_env(&vec![
                "
(def (fact n)
    (if (< n 1)
     1
     (* n (fact (- n 1)))))
",
                "(fact 5)"
            ]),
            Value::Num(120.0)
        );
    }

    #[test]
    fn define_func_nested_call() {
        assert_eq!(
            eval_str_env(&[
                "(def (double x) (* x 2))",
                "(def (quad x) (double (double x)))",
                "(quad 3)"
            ]),
            num(12.0)
        );
    }

    #[test]
    #[should_panic]
    fn define_func_wrong_arity() {
        eval_str_env(&["(def (f x) (+ x 1))", "(f 1 2)"]);
    }

    // --- if ---

    #[test]
    fn if_true_branch() {
        assert_eq!(eval_str("(if #t 1 2)"), num(1.0));
    }

    #[test]
    fn if_false_branch() {
        assert_eq!(eval_str("(if #f 1 2)"), num(2.0));
    }

    #[test]
    fn if_truthy_num() {
        assert_eq!(eval_str("(if 1 10 20)"), num(10.0));
    }

    #[test]
    fn if_falsy_zero() {
        assert_eq!(eval_str("(if 0 10 20)"), num(20.0));
    }

    #[test]
    fn if_falsy_nil() {
        assert_eq!(eval_str("(if nil 10 20)"), num(20.0));
    }

    #[test]
    fn if_condition_is_expression() {
        assert_eq!(eval_str("(if (< 1 2) 10 20)"), num(10.0));
    }

    #[test]
    fn if_only_evaluates_true_branch() {
        // if the false branch were evaluated it would panic (undefined symbol)
        eval_str_env(&["(def x 1)", "(if #t x undefined)"]);
    }

    #[test]
    fn if_only_evaluates_false_branch() {
        eval_str_env(&["(def x 1)", "(if #f undefined x)"]);
    }

    #[test]
    fn if_nested() {
        assert_eq!(eval_str("(if #t (if #f 1 2) 3)"), num(2.0));
    }

    #[test]
    #[should_panic]
    fn if_no_else_true() {
        assert_eq!(eval_str("(if #t 42)"), num(42.0));
    }

    #[test]
    #[should_panic]
    fn if_no_else_false() {
        assert_eq!(eval_str("(if #f 42)"), Value::Nil);
    }

    // --- cons ---

    #[test]
    fn cons_two_atoms() {
        assert_eq!(
            eval_str("(cons 1 2)"),
            Value::List(vec![num(1.0), num(2.0)])
        );
    }

    #[test]
    fn cons_with_nil_tail() {
        assert_eq!(
            eval_str("(cons 1 nil)"),
            Value::List(vec![num(1.0), Value::Nil])
        );
    }

    #[test]
    fn cons_with_list_tail() {
        // cons does not flatten — tail stays as a nested list
        assert_eq!(
            eval_str("(cons 1 '(2 3))"),
            Value::List(vec![num(1.0), Value::List(vec![num(2.0), num(3.0)])])
        );
    }

    #[test]
    #[should_panic]
    fn cons_wrong_arity_one() {
        eval_str("(cons 1)");
    }

    #[test]
    #[should_panic]
    fn cons_wrong_arity_three() {
        eval_str("(cons 1 2 3)");
    }

    // --- car ---

    #[test]
    fn car_of_cons() {
        assert_eq!(eval_str("(car (cons 1 2))"), num(1.0));
    }

    #[test]
    fn car_of_quoted_list() {
        assert_eq!(eval_str("(car '(10 20 30))"), num(10.0));
    }

    #[test]
    fn car_of_single_element_list() {
        assert_eq!(eval_str("(car '(42))"), num(42.0));
    }

    #[test]
    #[should_panic]
    fn car_of_nil() {
        eval_str("(car nil)");
    }

    #[test]
    #[should_panic]
    fn car_of_atom() {
        eval_str("(car 1)");
    }

    #[test]
    #[should_panic]
    fn car_wrong_arity() {
        eval_str("(car '(1) '(2))");
    }

    // --- cdr ---

    #[test]
    fn cdr_of_cons() {
        // cdr of (cons 1 2) is List([2]), not the atom 2
        assert_eq!(eval_str("(cdr (cons 1 2))"), Value::List(vec![num(2.0)]));
    }

    #[test]
    fn cdr_of_quoted_list() {
        assert_eq!(
            eval_str("(cdr '(1 2 3))"),
            Value::List(vec![num(2.0), num(3.0)])
        );
    }

    #[test]
    fn cdr_of_single_element_list_is_empty_list() {
        // cdr drops the head, leaving an empty Vec — not Nil
        assert_eq!(eval_str("(cdr '(1))"), Value::List(vec![]));
    }

    #[test]
    #[should_panic]
    fn cdr_of_nil() {
        eval_str("(cdr nil)");
    }

    #[test]
    #[should_panic]
    fn cdr_of_atom() {
        eval_str("(cdr 1)");
    }

    #[test]
    #[should_panic]
    fn cdr_wrong_arity() {
        eval_str("(cdr '(1) '(2))");
    }

    // --- car/cdr composition ---

    #[test]
    fn car_of_cdr_gives_second_element() {
        assert_eq!(eval_str("(car (cdr '(1 2 3)))"), num(2.0));
    }

    #[test]
    fn nested_cons_car_cdr() {
        // (cons 20 30) => List([20, 30])
        // (cons 10 ...) => List([10, List([20, 30])])
        // cdr         => List([20, 30])
        // car         => 20
        assert_eq!(eval_str("(car (cdr (cons 10 (cons 20 30))))"), num(20.0));
    }
}

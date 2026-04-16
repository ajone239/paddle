use super::*;
use crate::lexer::lex;
use crate::parser::parse_expr;

fn eval_str(s: &str) -> Value {
    let env = Env::default();
    let tokens = lex(s);
    let (expr, _) = parse_expr(&tokens).unwrap();
    let expr = lower(&expr);
    eval(&expr, Rc::new(RefCell::new(env))).unwrap()
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

    last.unwrap().unwrap()
}

fn num(n: f64) -> Value {
    Value::Num(n)
}

fn eval_err(s: &str) -> anyhow::Error {
    let env = Env::default();
    let tokens = lex(s);
    let (expr, _) = parse_expr(&tokens).unwrap();
    let expr = lower(&expr);
    eval(&expr, Rc::new(RefCell::new(env))).unwrap_err()
}

fn eval_env_err(exprs: &[&str]) -> anyhow::Error {
    let env = Rc::new(RefCell::new(Env::default()));
    for s in exprs {
        let tokens = lex(s);
        let (e, _) = parse_expr(&tokens).unwrap();
        let e = lower(&e);
        if let Err(err) = eval(&e, env.clone()) {
            return err;
        }
    }
    panic!("expected an error but all expressions succeeded");
}

mod atoms {
    use super::*;

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
}

mod addition {
    use super::*;

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
}
mod subtraction {
    use super::*;

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
}
mod multiplication {
    use super::*;

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
}
mod division {
    use super::*;

    #[test]
    fn div_two() {
        assert_eq!(eval_str("(/ 10 2)"), num(5.0));
    }

    #[test]
    fn div_three() {
        assert_eq!(eval_str("(/ 24 4 3)"), num(2.0));
    }
}
mod nesting {
    use super::*;

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
}
mod empty {
    use super::*;

    #[test]
    fn empty_list() {
        assert_eq!(eval_str("()"), Value::Nil);
    }
}
mod quote {
    use super::*;

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
}
mod evalenv {
    use super::*;

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
        let err = eval_err("x");
        assert_eq!(
            err.downcast_ref::<EvalError>(),
            Some(&EvalError::SymbolUndefined("x".into()))
        );
    }
}
mod env {
    use super::*;

    #[test]
    fn define_func_and_call() {
        assert_eq!(
            eval_str_env(&["(def (double x) (* x 2))", "(double 3)"]),
            num(6.0)
        );
    }

    #[test]
    fn define_func_scope() {
        let err = eval_env_err(&["(def (double x) (* x 2))", "(double 3)", "x"]);
        assert_eq!(
            err.downcast_ref::<EvalError>(),
            Some(&EvalError::SymbolUndefined("x".into()))
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
    fn define_func_wrong_arity() {
        let err = eval_env_err(&["(def (f x) (+ x 1))", "(f 1 2)"]);
        assert_eq!(
            err.downcast_ref::<EvalError>(),
            Some(&EvalError::BadFunctionArgCount(2, 1))
        );
    }
}

mod evalif {
    use super::*;

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
    fn if_no_else_true() {
        let err = eval_err("(if #t 42)");
        assert_eq!(err.downcast_ref::<EvalError>(), Some(&EvalError::BadIfArgs));
    }

    #[test]
    fn if_no_else_false() {
        let err = eval_err("(if #f 42)");
        assert_eq!(err.downcast_ref::<EvalError>(), Some(&EvalError::BadIfArgs));
    }
}
mod cons {
    use super::*;
    use crate::eval::env::BuiltinError;

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
    fn cons_wrong_arity_one() {
        let err = eval_err("(cons 1)");
        assert_eq!(
            err.downcast_ref::<BuiltinError>(),
            Some(&BuiltinError::WrongConsArgCount(1))
        );
    }

    #[test]
    fn cons_wrong_arity_three() {
        let err = eval_err("(cons 1 2 3)");
        assert_eq!(
            err.downcast_ref::<BuiltinError>(),
            Some(&BuiltinError::WrongConsArgCount(3))
        );
    }
}
mod car {
    use super::*;
    use crate::eval::env::BuiltinError;

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
    fn car_of_nil() {
        let err = eval_err("(car nil)");
        assert_eq!(
            err.downcast_ref::<BuiltinError>(),
            Some(&BuiltinError::WrongCarArgType)
        );
    }

    #[test]
    fn car_of_atom() {
        let err = eval_err("(car 1)");
        assert_eq!(
            err.downcast_ref::<BuiltinError>(),
            Some(&BuiltinError::WrongCarArgType)
        );
    }

    #[test]
    fn car_wrong_arity() {
        let err = eval_err("(car '(1) '(2))");
        assert_eq!(
            err.downcast_ref::<BuiltinError>(),
            Some(&BuiltinError::WrongCarArgCount(2))
        );
    }
}
mod cdr {
    use super::*;
    use crate::eval::env::BuiltinError;

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
    fn cdr_of_nil() {
        let err = eval_err("(cdr nil)");
        assert_eq!(
            err.downcast_ref::<BuiltinError>(),
            Some(&BuiltinError::WrongCdrArgType)
        );
    }

    #[test]
    fn cdr_of_atom() {
        let err = eval_err("(cdr 1)");
        assert_eq!(
            err.downcast_ref::<BuiltinError>(),
            Some(&BuiltinError::WrongCdrArgType)
        );
    }

    #[test]
    fn cdr_wrong_arity() {
        let err = eval_err("(cdr '(1) '(2))");
        assert_eq!(
            err.downcast_ref::<BuiltinError>(),
            Some(&BuiltinError::WrongCdrArgCount(2))
        );
    }
}

mod lambda {
    use super::*;

    #[test]
    fn call_immediately() {
        assert_eq!(eval_str("((lambda (x) (* x 2)) 5)"), num(10.0));
    }

    #[test]
    fn no_args() {
        assert_eq!(eval_str("((lambda () 42))"), num(42.0));
    }

    #[test]
    fn multi_arg() {
        assert_eq!(eval_str("((lambda (x y) (+ x y)) 3 4)"), num(7.0));
    }

    #[test]
    fn assign_and_call() {
        assert_eq!(
            eval_str_env(&["(def double (lambda (x) (* x 2)))", "(double 6)"]),
            num(12.0)
        );
    }

    #[test]
    fn multi_body_returns_last() {
        // body is (progn): intermediate exprs evaluated, last value returned
        assert_eq!(eval_str("((lambda (x) (+ x 1) (* x 2)) 3)"), num(6.0));
    }

    #[test]
    fn captures_outer_var() {
        assert_eq!(
            eval_str_env(&["(def y 10)", "((lambda (x) (+ x y)) 5)"]),
            num(15.0)
        );
    }

    #[test]
    fn args_do_not_leak() {
        // lambda arg `x` must not pollute the outer env
        assert_eq!(
            eval_str_env(&["(def x 99)", "((lambda (x) (* x 2)) 3)", "x"]),
            num(99.0)
        );
    }

    #[test]
    fn closure_captures_creation_env() {
        // classic adder: lambda closes over `n` from make-adder's call env
        assert_eq!(
            eval_str_env(&[
                "(def (make-adder n) (lambda (x) (+ x n)))",
                "(def add5 (make-adder 5))",
                "(add5 3)"
            ]),
            num(8.0)
        );
    }

    #[test]
    fn higher_order_apply() {
        // pass a lambda as an argument and call it
        assert_eq!(
            eval_str_env(&[
                "(def (apply-fn f x) (f x))",
                "(apply-fn (lambda (x) (* x x)) 4)"
            ]),
            num(16.0)
        );
    }

    #[test]
    fn wrong_arity() {
        let err = eval_err("((lambda (x) x) 1 2)");
        assert_eq!(
            err.downcast_ref::<EvalError>(),
            Some(&EvalError::BadFunctionArgCount(2, 1))
        );
    }

    #[test]
    fn alternate_syntax_backslash() {
        assert_eq!(eval_str("((.\\  (x) (+ x 1)) 9)"), num(10.0));
    }
}

mod builtin_errors {
    use super::*;
    use crate::eval::env::BuiltinError;

    #[test]
    fn car_empty_list() {
        // (car nil) hits WrongCarArgType; need a real empty list
        let err = eval_err("(car '())");
        assert_eq!(
            err.downcast_ref::<BuiltinError>(),
            Some(&BuiltinError::CarOnEmptyList)
        );
    }

    #[test]
    fn cdr_empty_list() {
        let err = eval_err("(cdr '())");
        assert_eq!(
            err.downcast_ref::<BuiltinError>(),
            Some(&BuiltinError::CdrOnEmptyList)
        );
    }

    #[test]
    fn expected_num_arg() {
        let err = eval_err(r#"(+ "foo" 1)"#);
        assert_eq!(
            err.downcast_ref::<BuiltinError>(),
            Some(&BuiltinError::ExpectedNumArg)
        );
    }

    #[test]
    fn minus_no_args() {
        let err = eval_err("(-)");
        assert_eq!(
            err.downcast_ref::<BuiltinError>(),
            Some(&BuiltinError::NoInitforMinus)
        );
    }

    #[test]
    fn div_no_args() {
        let err = eval_err("(/)");
        assert_eq!(
            err.downcast_ref::<BuiltinError>(),
            Some(&BuiltinError::NoInitforDiv)
        );
    }

    #[test]
    fn lt_too_few_args() {
        let err = eval_err("(< 1)");
        assert_eq!(
            err.downcast_ref::<BuiltinError>(),
            Some(&BuiltinError::BadLtArgCount(1))
        );
    }

    #[test]
    fn lt_bad_arg_types() {
        let err = eval_err(r#"(< "a" "b")"#);
        assert_eq!(
            err.downcast_ref::<BuiltinError>(),
            Some(&BuiltinError::BadLtArgTypes)
        );
    }
}

mod eval_errors {
    use super::*;

    #[test]
    fn bad_define_args() {
        // (def x) — missing value
        let err = eval_err("(def x)");
        assert_eq!(
            err.downcast_ref::<EvalError>(),
            Some(&EvalError::BadDefineArgs)
        );
    }

    #[test]
    fn bad_lambda_args() {
        // (lambda (x)) — missing body
        let err = eval_err("(lambda (x))");
        assert_eq!(
            err.downcast_ref::<EvalError>(),
            Some(&EvalError::BadLambdaArgs)
        );
    }

    #[test]
    fn bad_lambda_args_list() {
        // args must be a list, not a symbol
        let err = eval_err("(lambda x x)");
        assert_eq!(
            err.downcast_ref::<EvalError>(),
            Some(&EvalError::BadLambdaArgsList)
        );
    }

    #[test]
    fn bad_lambda_args_list_type() {
        // args list may only contain symbols
        let err = eval_err("(lambda (1) x)");
        assert_eq!(
            err.downcast_ref::<EvalError>(),
            Some(&EvalError::BadLambdaArgsListType)
        );
    }

    #[test]
    fn bad_define_head() {
        // head must be a symbol or list, not a number
        let err = eval_err("(def 5 x)");
        assert_eq!(
            err.downcast_ref::<EvalError>(),
            Some(&EvalError::BadDefineHead)
        );
    }

    #[test]
    fn bad_define_function_head() {
        // function def with empty head list has no name
        let err = eval_err("(def () x)");
        assert_eq!(
            err.downcast_ref::<EvalError>(),
            Some(&EvalError::BadDefineFunctionHead)
        );
    }

    #[test]
    fn bad_define_function_head_types() {
        // function head list may only contain symbols
        let err = eval_err("(def (1 x) x)");
        assert_eq!(
            err.downcast_ref::<EvalError>(),
            Some(&EvalError::BadDefineFunctionHeadTypes)
        );
    }
}

mod list {
    use super::*;

    #[test]
    fn list_empty() {
        assert_eq!(eval_str("(list)"), Value::List(vec![]));
    }

    #[test]
    fn list_single() {
        assert_eq!(eval_str("(list 1)"), Value::List(vec![num(1.0)]));
    }

    #[test]
    fn list_multiple() {
        assert_eq!(
            eval_str("(list 1 2 3)"),
            Value::List(vec![num(1.0), num(2.0), num(3.0)])
        );
    }

    #[test]
    fn list_evaluates_args() {
        assert_eq!(
            eval_str("(list (+ 1 1) (* 2 3))"),
            Value::List(vec![num(2.0), num(6.0)])
        );
    }

    #[test]
    fn list_mixed_types() {
        assert_eq!(
            eval_str("(list 1 #t nil)"),
            Value::List(vec![num(1.0), Value::Bool(true), Value::Nil])
        );
    }

    #[test]
    fn car_of_list() {
        assert_eq!(eval_str("(car (list 10 20 30))"), num(10.0));
    }

    #[test]
    fn cdr_of_list() {
        assert_eq!(
            eval_str("(cdr (list 1 2 3))"),
            Value::List(vec![num(2.0), num(3.0)])
        );
    }

    #[test]
    fn list_with_quoted_symbol() {
        assert_eq!(
            eval_str("(list 'a 'b)"),
            Value::List(vec![
                Value::Symbol("a".to_owned()),
                Value::Symbol("b".to_owned())
            ])
        );
    }

    #[test]
    fn list_with_quoted_list_arg() {
        assert_eq!(
            eval_str("(list '(1 2) 3)"),
            Value::List(vec![
                Value::List(vec![num(1.0), num(2.0)]),
                num(3.0)
            ])
        );
    }

    #[test]
    fn list_of_lists() {
        assert_eq!(
            eval_str("(list (list 1 2) (list 3 4))"),
            Value::List(vec![
                Value::List(vec![num(1.0), num(2.0)]),
                Value::List(vec![num(3.0), num(4.0)]),
            ])
        );
    }

    #[test]
    fn quote_of_list_call_suppresses_eval() {
        assert_eq!(
            eval_str("'(list 1 2)"),
            Value::List(vec![
                Value::Symbol("list".to_owned()),
                num(1.0),
                num(2.0)
            ])
        );
    }
}

mod cadr {
    use super::*;

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

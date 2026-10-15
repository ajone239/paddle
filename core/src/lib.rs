pub mod cursor;
pub mod env;
pub mod eval;
pub mod lexer;
pub mod lower;
pub mod parser;
pub mod span;
pub mod value;

pub fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub mod test_utils {
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        env::Env,
        eval::{EvalError, eval},
        lexer::lex_no_file,
        lower::lower,
        parser::parse_expr,
        span::Span,
        value::Value,
    };

    // `Value` and `EvalError` both carry a `Span` now, and both derive `PartialEq`
    // over it. Real spans from parsing never equal the `Span::default()` placeholder
    // tests build expected values with, so every comparison helper normalizes spans
    // away before handing values back to the tests.
    pub fn strip_value_span(v: &Value) -> Value {
        match v {
            Value::NoPrint => Value::NoPrint,
            Value::Nil(_) => Value::Nil(Span::default()),
            Value::Bool(b, _) => Value::Bool(*b, Span::default()),
            Value::Char(c, _) => Value::Char(*c, Span::default()),
            Value::Num(n, _) => Value::Num(*n, Span::default()),
            Value::Symbol(s, _) => Value::Symbol(s.clone(), Span::default()),
            Value::Form(f, _) => Value::Form(*f, Span::default()),
            Value::Str(s, _) => Value::Str(s.clone(), Span::default()),
            Value::Cons(pair, _) => Value::Cons(
                Rc::new((strip_value_span(&pair.0), strip_value_span(&pair.1))),
                Span::default(),
            ),
            Value::Builtin(f, _) => Value::Builtin(*f, Span::default()),
            Value::Func(f, _) => Value::Func(f.clone(), Span::default()),
            Value::Macro(m, _) => Value::Macro(m.clone(), Span::default()),
        }
    }

    fn strip_eval_error_span(e: EvalError) -> EvalError {
        match e {
            EvalError::BadRequireArgCount(n, _) => {
                EvalError::BadRequireArgCount(n, Span::default())
            }
            EvalError::BadRequireArgs(_) => EvalError::BadRequireArgs(Span::default()),
            EvalError::BadSetBangArgs(_) => EvalError::BadSetBangArgs(Span::default()),
            EvalError::BadDefineArgs(_) => EvalError::BadDefineArgs(Span::default()),
            EvalError::BadIfArgs(_) => EvalError::BadIfArgs(Span::default()),
            EvalError::BadLambdaArgs(_) => EvalError::BadLambdaArgs(Span::default()),
            EvalError::BadLambdaArgsList(_) => EvalError::BadLambdaArgsList(Span::default()),
            EvalError::SymbolUndefined(s, _) => EvalError::SymbolUndefined(s, Span::default()),
            EvalError::BadDefineHead(_) => EvalError::BadDefineHead(Span::default()),
            EvalError::BadLambdaArgsListType(_) => {
                EvalError::BadLambdaArgsListType(Span::default())
            }
            EvalError::BadFunctionArgCount(n, _) => {
                EvalError::BadFunctionArgCount(n, Span::default())
            }
            EvalError::BadDefineFunctionHead(_) => {
                EvalError::BadDefineFunctionHead(Span::default())
            }
            EvalError::BadDefineFunctionHeadTypes(_) => {
                EvalError::BadDefineFunctionHeadTypes(Span::default())
            }
            EvalError::EmptyPrognBody(_) => EvalError::EmptyPrognBody(Span::default()),
            EvalError::UnquoteOutsideQuasi(_) => EvalError::UnquoteOutsideQuasi(Span::default()),
            EvalError::VariadicArgsMustBeLast(_) => {
                EvalError::VariadicArgsMustBeLast(Span::default())
            }
            EvalError::BadCallableArgs(f, _) => EvalError::BadCallableArgs(f, Span::default()),
            EvalError::BadCallableArgsListType(f, _) => {
                EvalError::BadCallableArgsListType(f, Span::default())
            }
            EvalError::BadCallableHead(f, _) => EvalError::BadCallableHead(f, Span::default()),
            EvalError::BadCallableBodyArgs(f, _) => {
                EvalError::BadCallableBodyArgs(f, Span::default())
            }
        }
    }

    pub fn normalize_err(err: anyhow::Error) -> anyhow::Error {
        match err.downcast::<EvalError>() {
            Ok(e) => strip_eval_error_span(e).into(),
            Err(err) => err,
        }
    }

    pub fn eval_str(s: &str) -> Value {
        let env = Env::default();
        let tokens = lex_no_file(s);
        let (expr, _) = parse_expr(&tokens).unwrap();
        let expr = lower(&expr);
        let val = eval(expr, Rc::new(RefCell::new(env))).unwrap();
        strip_value_span(&val)
    }

    pub fn eval_str_env(exprs: &[&str]) -> Value {
        let env = Rc::new(RefCell::new(Env::default()));

        let mut last = None;

        for expr in exprs {
            let tokens = lex_no_file(expr);
            let (e, _) = parse_expr(&tokens).unwrap();
            let e = lower(&e);
            let val = eval(e, env.clone());
            last = Some(val);
        }

        strip_value_span(&last.unwrap().unwrap())
    }

    pub fn num(n: f64) -> Value {
        Value::Num(n, Span::default())
    }

    pub fn sym(s: &str) -> Value {
        Value::Symbol(s.into(), Span::default())
    }

    pub fn cons(head: Value, tail: Value) -> Value {
        Value::Cons(Rc::new((head, tail)), Span::default())
    }

    pub fn eval_err(s: &str) -> anyhow::Error {
        let env = Env::default();
        let tokens = lex_no_file(s);
        let (expr, _) = parse_expr(&tokens).unwrap();
        let expr = lower(&expr);
        let err = eval(expr, Rc::new(RefCell::new(env))).unwrap_err();
        normalize_err(err)
    }

    pub fn eval_env_err(exprs: &[&str]) -> anyhow::Error {
        let env = Rc::new(RefCell::new(Env::default()));
        for s in exprs {
            let tokens = lex_no_file(s);
            let (e, _) = parse_expr(&tokens).unwrap();
            let e = lower(&e);
            if let Err(err) = eval(e, env.clone()) {
                return normalize_err(err);
            }
        }
        panic!("expected an error but all expressions succeeded");
    }
}

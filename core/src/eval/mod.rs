mod form;
mod quasi;

use anyhow::{Context, Ok, Result};
use thiserror::Error;

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use crate::eval::form::eval_form;
use crate::{env::Env, value::Value};
use crate::{span::Span, value::Form};

#[derive(Debug, PartialEq, Error)]
pub enum EvalError {
    #[error("Require takes 2 arguments got {0} at {1}")]
    BadRequireArgCount(usize, Span),
    #[error("Require takes strings or symbols as args at {0}")]
    BadRequireArgs(Span),
    #[error("Too few arguments were provided to the set! statement at {0}")]
    BadSetBangArgs(Span),
    #[error("Too few arguments were provided to the define statement at {0}")]
    BadDefineArgs(Span),
    #[error("Too few arguments were provided to the if statement at {0}")]
    BadIfArgs(Span),
    #[error("Too few arguments were provided to the lambda statement at {0}")]
    BadLambdaArgs(Span),
    #[error("A list is required for lambda args at {0}")]
    BadLambdaArgsList(Span),
    #[error("Symbol [{0}] is undefined in current env. at {1}")]
    SymbolUndefined(String, Span),
    #[error("Symbol or list expected. at {0}")]
    BadDefineHead(Span),
    #[error("Lambda function args list must only be symbols at {0}")]
    BadLambdaArgsListType(Span),
    #[error("Function expected {0} args. at {1}")]
    BadFunctionArgCount(usize, Span),
    #[error("Function definition requires atleast a function name. at {0}")]
    BadDefineFunctionHead(Span),
    #[error("Function definition head may only contain symbols. at {0}")]
    BadDefineFunctionHeadTypes(Span),
    #[error("Progn body must have entries at {0}")]
    EmptyPrognBody(Span),
    #[error("Unquoute called outside of a quasiquote context at {0}")]
    UnquoteOutsideQuasi(Span),
    #[error("Variadic args must come last at {0}")]
    VariadicArgsMustBeLast(Span),
    #[error("No args provided for callable: {0:?} at {1}")]
    BadCallableArgs(Form, Span),
    #[error("bad types for callable: {0:?} at {1}")]
    BadCallableArgsListType(Form, Span),
    #[error("Need name for named callable: {0:?} at {1}")]
    BadCallableHead(Form, Span),
    #[error("No body for callable: {0:?} at {1}")]
    BadCallableBodyArgs(Form, Span),
}

enum Trampoline {
    Done(Value),
    Continue(Value, Rc<RefCell<Env>>),
}

impl Trampoline {
    fn unwrap(self) -> Value {
        match self {
            Trampoline::Done(value) => value,
            Trampoline::Continue(value, _) => value,
        }
    }
}

pub fn eval(ast: Value, env: Rc<RefCell<Env>>) -> Result<Value> {
    let mut current = (ast, env);

    loop {
        match eval_step(current.0, current.1)? {
            Trampoline::Done(v) => return Ok(v),
            Trampoline::Continue(body, nenv) => current = (body, nenv),
        }
    }
}

fn eval_step(ast: Value, env: Rc<RefCell<Env>>) -> Result<Trampoline> {
    let Value::Cons(pair, cons_span) = ast else {
        return match ast {
            Value::Symbol(atom, span) => {
                let val = resolve(&atom, span, env)?;
                Ok(Trampoline::Done(val))
            }
            _ => Ok(Trampoline::Done(ast.clone())),
        };
    };

    match pair.0 {
        Value::Nil(_) => Ok(Trampoline::Done(Value::Nil(Span::default()))),
        Value::Form(f, span) => eval_form(f, span, pair.1.clone(), env),
        _ => {
            let head = eval(pair.0.clone(), env.clone()).context(format!(
                "error evaling head of list [{}] at {}",
                pair.0.clone(),
                cons_span
            ))?;
            let tail = &pair.1;

            let (is_macro, info, fspan) = match head {
                Value::Macro(info, fspan) => (true, info, fspan),
                Value::Func(info, fspan) => (false, info, fspan),
                Value::Builtin(f, _) => {
                    let results = f.0(&tail
                        .to_cons_iter()
                        .map(|v| eval(v.clone(), env.clone()))
                        .collect::<Result<_, _>>()?)
                    .context(format!(
                        "error calling [{}] at {}",
                        pair.0.clone(),
                        cons_span
                    ))?;
                    return Ok(Trampoline::Done(results));
                }
                // TODO(austin.jones): understand or kill this.
                v => return Ok(Trampoline::Done(v.clone())),
            };

            let fenv = if is_macro {
                env.clone()
            } else {
                info.env.clone()
            };

            let nenv = setup_env(tail, cons_span, &info.args, is_macro, env.clone(), fenv)?;

            // TODO(austin.jones): understand or kill this.
            let body = info.body.deref().clone().set_span(fspan);
            if is_macro {
                let body = eval(body, nenv)?;
                // println!("{}", body);
                Ok(Trampoline::Continue(body, env.clone()))
            } else {
                Ok(Trampoline::Continue(body, nenv.clone()))
            }
        }
    }
}

fn resolve(atom: &str, span: Span, env: Rc<RefCell<Env>>) -> Result<Value> {
    env.borrow()
        .resolve(atom)
        .ok_or(EvalError::SymbolUndefined(atom.to_string(), span).into())
}

fn setup_env(
    tail: &Value,
    call_site_span: Span,
    fargs: &[String],
    is_macro: bool,
    old_env: Rc<RefCell<Env>>,
    new_env: Rc<RefCell<Env>>,
) -> Result<Rc<RefCell<Env>>> {
    let new_env = Rc::new(RefCell::new(Env::new_child(new_env.clone())));

    let varidx = fargs
        .iter()
        .position(|a| a.ends_with("..."))
        .unwrap_or(fargs.len());

    // + 1 deals with underflow of usize
    if varidx + 1 < fargs.len() {
        return Err(EvalError::VariadicArgsMustBeLast(call_site_span).into());
    }

    let mut citer = tail.to_cons_iter();
    for arg in fargs[..varidx].iter() {
        let val = citer
            .next()
            .ok_or(EvalError::BadFunctionArgCount(fargs.len(), call_site_span))?;

        let val = if is_macro {
            val.clone()
        } else {
            eval(val.clone(), old_env.clone())?
        };

        new_env.borrow_mut().define(arg, val.clone());
    }

    if varidx < fargs.len() {
        let rest = if is_macro {
            citer.into_cons_list()
        } else {
            citer
                .map(|val| eval(val.clone(), old_env.clone()))
                .collect::<Result<_, _>>()?
        };
        new_env.borrow_mut().define(&fargs[varidx], rest);
    } else if citer.next().is_some() {
        return Err(EvalError::BadFunctionArgCount(fargs.len(), call_site_span).into());
    }

    Ok(new_env)
}

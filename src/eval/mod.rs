pub mod env;
#[cfg(test)]
pub mod tests;
pub mod value;

use std::cell::RefCell;
use std::{ops::Deref, rc::Rc};

use anyhow::Result;
use thiserror::Error;

use crate::eval::env::Env;
use crate::eval::value::{Form, Value};
use crate::parser::Expr;

#[derive(Debug, PartialEq, Error)]
pub enum EvalError {
    #[error("Too few arguments were provided to the define statement")]
    BadDefineArgs,
    #[error("Too few arguments were provided to the if statement")]
    BadIfArgs,
    #[error("Too few arguments were provided to the lambda statement")]
    BadLambdaArgs,
    #[error("A list is required for lambda args")]
    BadLambdaArgsList,
    #[error("Symbol [{0}] is undefined in current env.")]
    SymbolUndefined(String),
    #[error("Symbol or list expected.")]
    BadDefineHead,
    #[error("Lambda function args list must only be symbols")]
    BadLambdaArgsListType,
    #[error("Function expected {0} args but got {1}.")]
    BadFunctionArgCount(usize, usize),
    #[error("Function definition requires atleast a function name.")]
    BadDefineFunctionHead,
    #[error("Function definition head may only contain symbols.")]
    BadDefineFunctionHeadTypes,
    #[error("PrognBodyMustHaveEntries")]
    EmptyPrognBody,
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

pub fn eval(ast: &Value, env: Rc<RefCell<Env>>) -> Result<Value> {
    match ast {
        Value::Symbol(atom) => resolve(atom, env),
        Value::List(list) if list.is_empty() => Ok(Value::Nil),
        Value::List(list) => {
            let head = &list[0];

            match head {
                Value::Form(Form::Quote) => {
                    return Ok(list[1].clone());
                }
                Value::Form(Form::Define) => {
                    if list.len() < 3 {
                        return Err(EvalError::BadDefineArgs.into());
                    }

                    define(&list[1], &list[2..], env)?;

                    return Ok(Value::Nil);
                }
                Value::Form(Form::If) => {
                    if list.len() < 4 {
                        return Err(EvalError::BadIfArgs.into());
                    }

                    let cond = &list[1];
                    let t_branch = &list[2];
                    let f_branch = &list[3];

                    let cond = eval(cond, env.clone())?;

                    return if cond.truthy() {
                        eval(t_branch, env)
                    } else {
                        eval(f_branch, env)
                    };
                }
                Value::Form(Form::Lambda) => {
                    if list.len() < 3 {
                        return Err(EvalError::BadLambdaArgs.into());
                    }

                    let Value::List(args) = &list[1] else {
                        return Err(EvalError::BadLambdaArgsList.into());
                    };

                    let args = args
                        .iter()
                        .map(|e| match e {
                            Value::Symbol(a) => Ok((*a).to_owned()),
                            _ => Err(EvalError::BadLambdaArgsListType),
                        })
                        .collect::<Result<Vec<String>, EvalError>>()?;

                    let tail = &list[2..];

                    let body = if tail.len() == 1 {
                        Rc::new(tail[0].clone())
                    } else {
                        Rc::new(Value::Progn(tail.to_vec()))
                    };
                    return Ok(Value::Lambda {
                        args,
                        body,
                        env: env.clone(),
                    });
                }
                _ => {}
            }

            let list = list
                .iter()
                .map(|v| eval(v, env.clone()))
                .collect::<Result<Vec<_>, _>>()?;
            apply(&list, env)
        }
        _ => Ok(ast.clone()),
    }
}

fn resolve(atom: &str, env: Rc<RefCell<Env>>) -> Result<Value> {
    match env.borrow().resolve(atom) {
        Some(val) => Ok(val),
        _ => Err(EvalError::SymbolUndefined(atom.to_string()).into()),
    }
}

fn apply(list: &[Value], env: Rc<RefCell<Env>>) -> Result<Value> {
    let args = &list[1..];

    let (fargs, body, new_env) = match &list[0] {
        Value::Lambda {
            args: fargs,
            body,
            env: lenv,
        } => {
            let lenv = Rc::new(RefCell::new(Env::new_child(lenv.clone())));
            (fargs, body, lenv)
        }
        Value::Func {
            name: _,
            args: fargs,
            body,
        } => {
            let env = Rc::new(RefCell::new(Env::new_child(env)));
            (fargs, body, env)
        }
        Value::Builtin(f, _) => return f.0(args),
        v => return Ok(v.clone()),
    };

    if fargs.len() != args.len() {
        return Err(EvalError::BadFunctionArgCount(args.len(), fargs.len()).into());
    }

    for (arg, val) in fargs.iter().zip(args) {
        new_env.borrow_mut().define(arg, val.clone());
    }

    // eval the body with the new env
    // return the value
    match body.deref() {
        Value::Progn(body) => {
            if body.is_empty() {
                return Err(EvalError::EmptyPrognBody.into());
            }
            for b in &body[..body.len() - 1] {
                eval(b, new_env.clone())?;
            }
            eval(
                body.last().expect("progn body can't be empty"),
                new_env.clone(),
            )
        }
        _ => eval(body, new_env.clone()),
    }
}

fn define(head: &Value, tail: &[Value], env: Rc<RefCell<Env>>) -> Result<()> {
    match head {
        Value::Symbol(atom) => {
            let value = eval(&tail[0], env.clone())?;
            env.borrow_mut().define(atom, value);
        }
        Value::List(exprs) => {
            let args_list = exprs
                .iter()
                .map(|e| match e {
                    Value::Symbol(a) => Ok((*a).to_owned()),
                    _ => Err(EvalError::BadDefineFunctionHeadTypes),
                })
                .collect::<Result<Vec<String>, EvalError>>()?;

            if args_list.is_empty() {
                return Err(EvalError::BadDefineFunctionHead.into());
            }

            let name = &args_list[0];
            let args = if args_list.len() == 1 {
                vec![]
            } else {
                args_list[1..].to_vec()
            };

            let body = if tail.len() == 1 {
                Rc::new(tail[0].clone())
            } else {
                Rc::new(Value::Progn(tail.to_vec()))
            };

            let func = Value::Func {
                name: name.clone(),
                args,
                body,
            };

            env.borrow_mut().define(name, func);
        }
        _ => return Err(EvalError::BadDefineHead.into()),
    };

    Ok(())
}

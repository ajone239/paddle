use std::cell::RefCell;
use std::{ops::Deref, rc::Rc};

use anyhow::{Ok, Result};

use crate::cursor::process_file;
use crate::eval::EvalError;
use crate::eval::env::Env;
use crate::eval::value::{Form, Value};

pub fn eval(ast: &Value, env: Rc<RefCell<Env>>) -> Result<Value> {
    match ast {
        Value::Symbol(atom) => resolve(atom, env),
        Value::List(list) if list.is_empty() => Ok(Value::Nil),
        Value::List(list) => match &list[0] {
            Value::Form(f) => eval_form(*f, list, env),
            _ => apply(&list, env),
        },
        _ => Ok(ast.clone()),
    }
}

fn resolve(atom: &str, env: Rc<RefCell<Env>>) -> Result<Value> {
    env.borrow()
        .resolve(atom)
        .ok_or(EvalError::SymbolUndefined(atom.to_string()).into())
}

fn eval_form(form: Form, list: &[Value], env: Rc<RefCell<Env>>) -> Result<Value> {
    match form {
        Form::Quote => Ok(list[1].clone()),
        Form::Require => {
            if list.len() != 2 {
                return Err(EvalError::BadRequireArgCount(list.len()).into());
            }

            let file_name = match &list[1] {
                Value::Str(atom) | Value::Symbol(atom) => atom,
                _ => {
                    return Err(EvalError::BadRequireArgs.into());
                }
            };

            process_file(file_name.into(), env.clone())?;

            Ok(Value::Nil)
        }
        Form::Eval => {
            let val = eval(&list[1], env.clone())?;
            eval(&val, env.clone())
        }
        Form::Define => {
            if list.len() < 3 {
                return Err(EvalError::BadDefineArgs.into());
            }

            define(&list[1], &list[2..], env)?;

            Ok(Value::Nil)
        }
        Form::If => {
            if list.len() < 4 {
                return Err(EvalError::BadIfArgs.into());
            }

            let cond = &list[1];
            let t_branch = &list[2];
            let f_branch = &list[3];

            let cond = eval(cond, env.clone())?;

            if cond.truthy() {
                eval(t_branch, env)
            } else {
                eval(f_branch, env)
            }
        }
        Form::Lambda => {
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
                    _ => Err(EvalError::BadLambdaArgsListType.into()),
                })
                .collect::<Result<Vec<String>, _>>()?;

            let tail = &list[2..];

            let body = if tail.len() == 1 {
                Rc::new(tail[0].clone())
            } else {
                Rc::new(Value::Progn(tail.to_vec()))
            };

            let lambda = Value::Lambda {
                args,
                body,
                env: env.clone(),
            };

            Ok(lambda)
        }
    }
}

fn apply(list: &[Value], env: Rc<RefCell<Env>>) -> Result<Value> {
    let list = list
        .iter()
        .map(|v| eval(v, env.clone()))
        .collect::<Result<Vec<_>, _>>()?;

    let args = &list[1..];

    let (fargs, body, new_env) = match &list[0] {
        Value::Lambda {
            args: fargs,
            body,
            env,
        } => {
            let env = Rc::new(RefCell::new(Env::new_child(env.clone())));
            (fargs, body, env)
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
        return Err(EvalError::BadFunctionArgCount(fargs.len(), args.len()).into());
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
                    _ => Err(EvalError::BadDefineFunctionHeadTypes.into()),
                })
                .collect::<Result<Vec<String>, _>>()?;

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

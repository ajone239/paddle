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

fn quasi_quote_eval(ast: &Value, env: Rc<RefCell<Env>>) -> Result<Value> {
    match ast {
        Value::List(values) if values.is_empty() => Ok(ast.clone()),
        Value::List(values) if matches!(values[0], Value::Form(Form::UnQuote)) => {
            eval(&values[1], env)
        }
        Value::List(values) => Ok(Value::List(
            values
                .iter()
                .map(|v| quasi_quote_eval(v, env.clone()))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        _ => Ok(ast.clone()),
    }
}

fn eval_form(form: Form, list: &[Value], env: Rc<RefCell<Env>>) -> Result<Value> {
    match form {
        Form::Quote => Ok(list[1].clone()),
        Form::QuasiQuote => quasi_quote_eval(&list[1], env),
        Form::UnQuote => Err(EvalError::UnquoteOutsideQuasi.into()),
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
        Form::Progn => {
            let body = &list[1..];

            if body.is_empty() {
                return Err(EvalError::EmptyPrognBody.into());
            }
            for b in &body[..body.len() - 1] {
                eval(b, env.clone())?;
            }
            eval(body.last().expect("progn body can't be empty"), env.clone())
        }
        Form::Eval => {
            let val = eval(&list[1], env.clone())?;
            eval(&val, env.clone())
        }
        Form::DefineMacro | Form::Define => {
            if list.len() < 3 {
                return Err(EvalError::BadDefineArgs.into());
            }

            define(&form, &list[1], &list[2..], env)?;

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
    let mut liter = list.iter().map(|v| eval(v, env.clone()));

    let head = liter.next().expect("can't call this on empty list")?;

    let (fargs, body, new_env) = match &head {
        Value::Lambda {
            args: fargs,
            body,
            env,
        } => {
            let env = Rc::new(RefCell::new(Env::new_child(env.clone())));
            (fargs, body, env)
        }
        Value::Macro {
            name: _,
            args: fargs,
            body,
        }
        | Value::Func {
            name: _,
            args: fargs,
            body,
        } => {
            let env = Rc::new(RefCell::new(Env::new_child(env.clone())));
            (fargs, body, env)
        }
        Value::Builtin(f, _) => {
            let args = liter.collect::<Result<Vec<_>, _>>()?;
            return f.0(&args);
        }
        v => return Ok(v.clone()),
    };

    let is_macro = matches!(head, Value::Macro { .. });

    let args = if is_macro {
        &list[1..]
    } else {
        &liter.collect::<Result<Vec<_>, _>>()?
    };

    let variadic = fargs.last().unwrap_or(&"".to_string()).ends_with("...");

    if variadic {
        let non_var_arg_count = fargs.len() - 1;

        if args.len() < non_var_arg_count {
            return Err(EvalError::BadFunctionArgCount(fargs.len(), args.len()).into());
        }

        for i in 0..non_var_arg_count {
            let arg = &fargs[i];
            let val = &args[i];
            new_env.borrow_mut().define(arg, val.clone());
        }

        let lfarg = &fargs[non_var_arg_count];
        let largs = &args[non_var_arg_count..];
        let largs = Value::List(largs.to_vec());

        new_env.borrow_mut().define(lfarg, largs);
    } else {
        if fargs.len() != args.len() {
            return Err(EvalError::BadFunctionArgCount(fargs.len(), args.len()).into());
        }

        for (arg, val) in fargs.iter().zip(args) {
            new_env.borrow_mut().define(arg, val.clone());
        }
    }

    // eval the body with the new env
    // return the value
    let rv = match body.deref() {
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
        _ => eval(&body, new_env.clone()),
    };

    if is_macro {
        eval(&rv?, env.clone())
    } else {
        rv
    }
}

fn define(form: &Form, head: &Value, tail: &[Value], env: Rc<RefCell<Env>>) -> Result<()> {
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

            let proc = match form {
                Form::Define => Value::Func {
                    name: name.clone(),
                    args,
                    body,
                },
                Form::DefineMacro => Value::Macro {
                    name: name.clone(),
                    args,
                    body,
                },
                _ => unreachable!("{:?} should only be able to be define or definemacro", form),
            };

            env.borrow_mut().define(name, proc);
        }
        _ => return Err(EvalError::BadDefineHead.into()),
    };

    Ok(())
}

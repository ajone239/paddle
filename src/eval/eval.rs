use std::cell::RefCell;
use std::{ops::Deref, rc::Rc};

use anyhow::{Ok, Result, bail};

use crate::cursor::process_file;
use crate::eval::EvalError;
use crate::eval::env::Env;
use crate::eval::value::{Form, Value};

pub fn eval(ast: &Value, env: &Rc<RefCell<Env>>) -> Result<Value> {
    match ast {
        Value::Symbol(atom) => resolve(atom, env),
        Value::Cons(pair) => match pair.0 {
            Value::Nil => Ok(Value::Nil),
            Value::Form(f) => eval_form(f, &pair.1, env),
            _ => apply(&pair.0, &pair.1, env),
        },
        _ => Ok(ast.clone()),
    }
}

fn resolve(atom: &str, env: &Rc<RefCell<Env>>) -> Result<Value> {
    env.borrow()
        .resolve(atom)
        .ok_or(EvalError::SymbolUndefined(atom.to_string()).into())
}

fn quasi_quote_eval(ast: &Value, env: &Rc<RefCell<Env>>) -> Result<Value> {
    match ast {
        Value::Cons(pair) => match pair.0 {
            Value::Nil => Ok(ast.clone()),
            Value::Form(Form::UnQuote) => eval(&pair.1, env),
            _ => {
                let new_head = quasi_quote_eval(&pair.0, env)?;
                let new_tail = quasi_quote_eval(&pair.1, env)?;

                Ok(Value::Cons(Rc::new((new_head, new_tail))))
            }
        },
        _ => Ok(ast.clone()),
    }
}

fn eval_form(form: Form, tail: &Value, env: &Rc<RefCell<Env>>) -> Result<Value> {
    match form {
        Form::Quote => {
            let Value::Cons(tailtail) = tail else {
                unreachable!("this is how quote is formed")
            };
            Ok(tailtail.0.clone())
        }
        Form::QuasiQuote => {
            let Value::Cons(tailtail) = tail else {
                unreachable!("this is how quasiquote is formed")
            };
            quasi_quote_eval(&tailtail.0, env)
        }
        Form::UnQuote => Err(EvalError::UnquoteOutsideQuasi.into()),
        Form::Require => {
            let mut list = tail.to_cons_iter();

            let file_name = list.next().ok_or(EvalError::BadRequireArgCount(0))?;

            let file_name = match file_name {
                Value::Str(atom) | Value::Symbol(atom) => atom,
                _ => {
                    return Err(EvalError::BadRequireArgs.into());
                }
            };

            if list.next().is_some() {
                return Err(EvalError::BadRequireArgCount(2).into());
            }

            process_file(file_name.into(), env)?;

            Ok(Value::Nil)
        }
        Form::Progn => {
            let mut body = tail.to_cons_iter().peekable();

            while let Some(b) = body.next() {
                let val = eval(b, env)?;

                if body.peek().is_none() {
                    return Ok(val);
                }
            }

            bail!("progn body can't be empty")
        }
        Form::Eval => {
            let val = eval(tail, env)?;
            eval(&val, env)
        }
        Form::DefineMacro | Form::Define => {
            define(&form, &tail, env)?;

            Ok(Value::Nil)
        }
        Form::If => {
            let mut list = tail.to_cons_iter();

            let cond = list.next().ok_or(EvalError::BadIfArgs)?;
            let t_branch = list.next().ok_or(EvalError::BadIfArgs)?;
            let f_branch = list.next().ok_or(EvalError::BadIfArgs)?;

            if list.next().is_some() {
                return Err(EvalError::BadIfArgs.into());
            }

            let cond = eval(cond, env)?;

            if cond.truthy() {
                eval(t_branch, env)
            } else {
                eval(f_branch, env)
            }
        }
        Form::Lambda => {
            let mut list = tail.to_cons_iter();

            let arg_head = list.next().ok_or(EvalError::BadLambdaArgs)?;

            // TODO(ajone239): fix empty list semantics
            if !matches!(arg_head, Value::Cons(_) | Value::Nil) {
                return Err(EvalError::BadLambdaArgsList.into());
            }

            let args = arg_head
                .to_cons_iter()
                .map(|e| match e {
                    Value::Symbol(a) => Ok((*a).to_owned()),
                    _ => Err(EvalError::BadLambdaArgsListType.into()),
                })
                .collect::<Result<Vec<String>, _>>()?;

            // TODO(ajone239): kill this clone
            let tail: Vec<_> = list.map(|v| v.clone()).collect();

            if tail.is_empty() {
                return Err(EvalError::BadLambdaArgs.into());
            }

            let body = if tail.len() == 1 {
                Rc::new(tail[0].clone())
            } else {
                Rc::new(Value::Progn(tail))
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

fn apply(head: &Value, tail: &Value, env: &Rc<RefCell<Env>>) -> Result<Value> {
    let head = eval(head, env)?;
    let cons_iter = tail.to_cons_iter();
    let clen = tail.to_cons_iter().len();

    let liter = tail.to_cons_iter().map(|v| eval(v, env));

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
            return f.0(&liter.collect::<Result<_, _>>()?);
        }
        v => return Ok(v.clone()),
    };

    let is_macro = matches!(head, Value::Macro { .. });

    if clen != fargs.len() {
        return Err(EvalError::BadFunctionArgCount(fargs.len()).into());
    }

    if is_macro {
        for (arg, val) in fargs.iter().zip(cons_iter) {
            // TODO(ajone239): clone arg
            new_env.borrow_mut().define(arg, val.clone());
        }
    } else {
        for (arg, val) in fargs.iter().zip(liter) {
            new_env.borrow_mut().define(arg, val?);
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
                eval(b, &new_env)?;
            }
            eval(body.last().expect("progn body can't be empty"), &new_env)
        }
        _ => eval(&body, &new_env),
    };

    if is_macro { eval(&rv?, env) } else { rv }
}

fn define(form: &Form, body: &Value, env: &Rc<RefCell<Env>>) -> Result<()> {
    let mut list = body.to_cons_iter();
    let head = list.next().ok_or(EvalError::BadDefineArgs)?;

    match head {
        Value::Symbol(atom) => {
            let tail = list.next().ok_or(EvalError::BadDefineArgs)?;

            if list.next().is_some() {
                return Err(EvalError::BadDefineArgs.into());
            }
            let value = eval(&tail, env)?;
            env.borrow_mut().define(atom, value);
        }
        Value::Cons(_) => {
            let args_list = head
                .to_cons_iter()
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

            let tail: Vec<_> = list.map(|v| v.to_owned()).collect();

            let body = if tail.len() == 1 {
                Rc::new(tail[0].clone())
            } else {
                Rc::new(Value::Progn(tail))
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

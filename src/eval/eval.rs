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
        Value::Cons(pair) => match pair.0 {
            Value::Nil => Ok(Value::Nil),
            Value::Form(f) => eval_form(f, &pair.0, &pair.1, env),
            _ => apply(&pair.0, &pair.1, env),
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
        Value::Cons(pair) => match pair.0 {
            Value::Nil => Ok(ast.clone()),
            Value::Form(Form::UnQuote) => eval(&pair.1, env),
            _ => {
                let new_head = quasi_quote_eval(&pair.0, env.clone())?;
                let new_tail = quasi_quote_eval(&pair.1, env.clone())?;

                Ok(Value::Cons(Rc::new((new_head, new_tail))))
            }
        },
        _ => Ok(ast.clone()),
    }
}

fn eval_form(form: Form, head: &Value, tail: &Value, env: Rc<RefCell<Env>>) -> Result<Value> {
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
            let list = tail.to_vec();
            if list.len() != 1 {
                return Err(EvalError::BadRequireArgCount(list.len()).into());
            }

            let file_name = match &list[0] {
                Value::Str(atom) | Value::Symbol(atom) => atom,
                _ => {
                    return Err(EvalError::BadRequireArgs.into());
                }
            };

            process_file(file_name.into(), env.clone())?;

            Ok(Value::Nil)
        }
        Form::Progn => {
            let body = tail.to_vec();

            if body.is_empty() {
                return Err(EvalError::EmptyPrognBody.into());
            }
            for b in &body[..body.len() - 1] {
                eval(b, env.clone())?;
            }
            eval(body.last().expect("progn body can't be empty"), env.clone())
        }
        Form::Eval => {
            let val = eval(tail, env.clone())?;
            eval(&val, env.clone())
        }
        Form::DefineMacro | Form::Define => {
            let list = tail.to_vec();
            if list.len() < 2 {
                return Err(EvalError::BadDefineArgs.into());
            }

            define(&form, &list[0], &list[1..], env)?;

            Ok(Value::Nil)
        }
        Form::If => {
            let list = tail.to_vec();
            println!("{}", tail);
            if list.len() < 3 {
                return Err(EvalError::BadIfArgs.into());
            }

            let cond = &list[0];
            let t_branch = &list[1];
            let f_branch = &list[2];

            let cond = eval(cond, env.clone())?;

            if cond.truthy() {
                eval(t_branch, env)
            } else {
                eval(f_branch, env)
            }
        }
        Form::Lambda => {
            let list = tail.to_vec();
            if list.len() < 2 {
                return Err(EvalError::BadLambdaArgs.into());
            }

            if !matches!(&list[0], Value::Cons(_)) {
                return Err(EvalError::BadLambdaArgsList.into());
            }

            let args = list[0]
                .to_vec()
                .iter()
                .map(|e| match e {
                    Value::Symbol(a) => Ok((*a).to_owned()),
                    _ => Err(EvalError::BadLambdaArgsListType.into()),
                })
                .collect::<Result<Vec<String>, _>>()?;

            let tail = &list[1..];

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

fn apply(head: &Value, tail: &Value, env: Rc<RefCell<Env>>) -> Result<Value> {
    let head = eval(head, env.clone())?;
    let list = tail.to_vec();

    let liter = list.iter().map(|v| eval(v, env.clone()));

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
            return f.0(&Value::to_cons_list(args));
        }
        v => return Ok(v.clone()),
    };

    let is_macro = matches!(head, Value::Macro { .. });

    let args = if is_macro {
        list
    } else {
        liter.collect::<Result<Vec<_>, _>>()?
    };

    if fargs.len() != args.len() {
        return Err(EvalError::BadFunctionArgCount(fargs.len(), args.len()).into());
    }

    for (arg, val) in fargs.iter().zip(args) {
        new_env.borrow_mut().define(arg, val.clone());
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
        Value::Cons(_) => {
            let args_list = head
                .to_vec()
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

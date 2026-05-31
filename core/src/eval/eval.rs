use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use anyhow::{Ok, Result, bail};

use crate::cursor::process_file;
use crate::eval::{
    EvalError,
    env::Env,
    value::{Form, Value},
};

enum Trampoline {
    Done(Value),
    Continue(Value, Rc<RefCell<Env>>),
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
    let Value::Cons(pair) = ast else {
        return match ast {
            Value::Symbol(atom) => {
                let val = resolve(&atom, env)?;
                Ok(Trampoline::Done(val))
            }
            _ => Ok(Trampoline::Done(ast.clone())),
        };
    };

    match pair.0 {
        Value::Nil => Ok(Trampoline::Done(Value::Nil)),
        Value::Form(f) => eval_form(f, pair.1.clone(), env),
        _ => {
            let head = eval(pair.0.clone(), env.clone())?;
            let is_macro = matches!(head, Value::Macro { .. });
            let tail = &pair.1;

            let (body, args, fenv) = match head {
                Value::Func {
                    name: _,
                    body,
                    args,
                    env,
                }
                | Value::Lambda { env, body, args } => (body, args, env.clone()),
                Value::Macro {
                    name: _,
                    body,
                    args,
                } => (body, args, env.clone()),
                Value::Builtin(f, _) => {
                    let results = f.0(&tail
                        .to_cons_iter()
                        .map(|v| eval(v.clone(), env.clone()))
                        .collect::<Result<_, _>>()?)?;
                    return Ok(Trampoline::Done(results));
                }
                v => return Ok(Trampoline::Done(v.clone())),
            };

            let nenv = setup_env(tail, &args, is_macro, env.clone(), fenv)?;

            let body = body.deref().clone();
            if is_macro {
                let body = eval(body, nenv)?;
                Ok(Trampoline::Continue(body, env.clone()))
            } else {
                Ok(Trampoline::Continue(body, nenv.clone()))
            }
        }
    }
}

fn resolve(atom: &str, env: Rc<RefCell<Env>>) -> Result<Value> {
    env.borrow()
        .resolve(atom)
        .ok_or(EvalError::SymbolUndefined(atom.to_string()).into())
}

fn quasi_quote_eval(ast: Value, env: Rc<RefCell<Env>>) -> Result<Value> {
    match ast {
        Value::Cons(ref pair) => match pair.0 {
            // TODO(ajone239): this can cause a weird bug between quote and quasi quote
            Value::Nil => Ok(ast.clone()),
            Value::Form(Form::UnQuote) => eval(pair.1.clone(), env),
            _ => {
                let new_head = quasi_quote_eval(pair.0.clone(), env.clone())?;
                let new_tail = quasi_quote_eval(pair.1.clone(), env.clone())?;

                Ok(Value::Cons(Rc::new((new_head, new_tail))))
            }
        },
        _ => Ok(ast.clone()),
    }
}

fn eval_form(form: Form, tail: Value, env: Rc<RefCell<Env>>) -> Result<Trampoline> {
    match form {
        Form::Quote => {
            let Value::Cons(tailtail) = tail else {
                unreachable!("this is how quote is formed: {}", tail)
            };
            Ok(Trampoline::Done(tailtail.0.clone()))
        }
        Form::QuasiQuote => {
            let Value::Cons(tailtail) = tail else {
                unreachable!("this is how quasiquote is formed")
            };
            let qexpr = quasi_quote_eval(tailtail.0.clone(), env)?;
            Ok(Trampoline::Done(qexpr))
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

            process_file(file_name.to_string().into(), env)?;

            Ok(Trampoline::Done(Value::Nil))
        }
        Form::Progn => {
            let mut body = tail.to_cons_iter().peekable();

            while let Some(b) = body.next() {
                if body.peek().is_none() {
                    return Ok(Trampoline::Continue(b.clone(), env.clone()));
                }
                let _ = eval(b.clone(), env.clone())?;
            }

            bail!("progn body can't be empty")
        }
        Form::Eval => {
            let val = eval(tail, env.clone())?;
            Ok(Trampoline::Continue(val, env.clone()))
        }
        Form::DefineMacro | Form::Define => {
            define(&form, &tail, env)?;

            Ok(Trampoline::Done(Value::Nil))
        }
        Form::If => {
            let mut list = tail.to_cons_iter();

            let cond = list.next().ok_or(EvalError::BadIfArgs)?;
            let t_branch = list.next().ok_or(EvalError::BadIfArgs)?;
            let f_branch = list.next().ok_or(EvalError::BadIfArgs)?;

            if list.next().is_some() {
                return Err(EvalError::BadIfArgs.into());
            }

            let cond = eval(cond.clone(), env.clone())?;

            if cond.truthy() {
                Ok(Trampoline::Continue(t_branch.clone(), env))
            } else {
                Ok(Trampoline::Continue(f_branch.clone(), env))
            }
        }
        Form::Lambda => {
            let (_, args, body) = make_callable(&form, &tail)?;

            let lambda = Value::Lambda {
                args,
                body,
                env: env.clone(),
            };

            Ok(Trampoline::Done(lambda))
        }
    }
}

fn setup_env(
    tail: &Value,
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
        return Err(EvalError::VariadicArgsMustBeLast.into());
    }

    let mut citer = tail.to_cons_iter();
    for arg in fargs[..varidx].iter() {
        let val = citer
            .next()
            .ok_or(EvalError::BadFunctionArgCount(fargs.len()))?;

        let val = if is_macro {
            val.clone()
        } else {
            eval(val.clone(), old_env.clone())?
        };

        new_env.borrow_mut().define(arg, val.clone());
    }

    if varidx < fargs.len() {
        let rest = if is_macro {
            citer.into_cons_list().clone()
        } else {
            citer
                .map(|val| eval(val.clone(), old_env.clone()))
                .collect::<Result<_, _>>()?
        };
        new_env.borrow_mut().define(&fargs[varidx], rest);
    } else if citer.next().is_some() {
        return Err(EvalError::BadFunctionArgCount(fargs.len()).into());
    }

    Ok(new_env)
}

fn define(form: &Form, body: &Value, env: Rc<RefCell<Env>>) -> Result<()> {
    let mut list = body.to_cons_iter();
    let head = list.next().ok_or(EvalError::BadDefineArgs)?;

    match head {
        Value::Symbol(atom) => {
            let tail = list.next().ok_or(EvalError::BadDefineArgs)?;

            if list.next().is_some() {
                return Err(EvalError::BadDefineArgs.into());
            }
            let value = eval(tail.clone(), env.clone())?;
            env.borrow_mut().define(atom, value);
        }
        Value::Cons(_) => {
            let (name, args, body) = make_callable(form, body)?;

            let Some(name) = name else {
                unreachable!();
            };
            let tag = name.clone();

            let proc = match form {
                Form::Define => Value::Func {
                    name,
                    args,
                    body,
                    env: env.clone(),
                },
                Form::DefineMacro => Value::Macro { name, args, body },
                _ => unreachable!("should only get here from define or definemacro"),
            };

            env.borrow_mut().define(tag.as_str(), proc);
        }
        _ => return Err(EvalError::BadDefineHead.into()),
    };

    Ok(())
}

type CallableInfo = (Option<String>, Vec<String>, Rc<Value>);
fn make_callable(form: &Form, body: &Value) -> Result<CallableInfo> {
    let mut list = body.to_cons_iter();
    let head = list.next().ok_or(EvalError::BadCallableArgs(*form))?;

    if !matches!(head, Value::Cons(_) | Value::Nil) {
        return Err(EvalError::BadCallableArgs(*form).into());
    }

    let args_list = head
        .to_cons_iter()
        .map(|e| match e {
            Value::Symbol(a) => Ok(a.to_string()),
            _ => Err(EvalError::BadCallableArgsListType(*form).into()),
        })
        .collect::<Result<Vec<String>, _>>()?;

    let (name, args) = if matches!(form, Form::Lambda) {
        (None, args_list)
    } else {
        if args_list.is_empty() {
            return Err(EvalError::BadCallableHead(*form).into());
        }
        (Some(args_list[0].to_owned()), args_list[1..].to_vec())
    };

    if list.is_empty() {
        return Err(EvalError::BadCallableBodyArgs(*form).into());
    }

    let body = Rc::new(Value::Cons(Rc::new((
        Value::Form(Form::Progn),
        list.into_cons_list().clone(),
    ))));

    Ok((name, args, body))
}

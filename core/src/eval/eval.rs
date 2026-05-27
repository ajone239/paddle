use core::panic;
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

const OLD: bool = false;
pub fn eval(ast: &Value, env: Rc<RefCell<Env>>) -> Result<Value> {
    if OLD {
        old_eval(ast, env)
    } else {
        new_eval(ast.clone(), env)
    }
}

enum Trampoline {
    Done(Value),
    Continue(Value, Rc<RefCell<Env>>),
}

pub fn new_eval(ast: Value, env: Rc<RefCell<Env>>) -> Result<Value> {
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
        Value::Form(Form::Progn) => {
            let mut body = pair.1.to_cons_iter().peekable();

            while let Some(b) = body.next() {
                if body.peek().is_none() {
                    return Ok(Trampoline::Continue(b.clone(), env.clone()));
                }
                let _ = eval(b, env.clone())?;
            }

            bail!("progn body can't be empty")
        }
        Value::Form(Form::Quote) => {
            let Value::Cons(ref tailtail) = pair.1 else {
                unreachable!("this is how quote is formed")
            };
            Ok(Trampoline::Done(tailtail.0.clone()))
        }
        Value::Form(Form::QuasiQuote) => {
            let Value::Cons(ref tailtail) = pair.1 else {
                unreachable!("this is how quasiquote is formed")
            };
            let qexpr = quasi_quote_eval(&tailtail.0, env)?;
            Ok(Trampoline::Done(qexpr))
        }
        Value::Form(Form::UnQuote) => Err(EvalError::UnquoteOutsideQuasi.into()),
        Value::Form(Form::Require) => {
            let mut list = pair.1.to_cons_iter();

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

            Ok(Trampoline::Done(Value::Nil))
        }
        Value::Form(Form::Eval) => {
            let val = eval(&pair.1, env.clone())?;
            Ok(Trampoline::Continue(val, env.clone()))
        }
        Value::Form(Form::DefineMacro | Form::Define) => {
            let Value::Form(form) = pair.0 else {
                panic!("it's the pattern");
            };
            define(&form, &pair.1, env)?;

            Ok(Trampoline::Done(Value::Nil))
        }
        Value::Form(Form::If) => {
            let mut list = pair.1.to_cons_iter();

            let cond = list.next().ok_or(EvalError::BadIfArgs)?;
            let t_branch = list.next().ok_or(EvalError::BadIfArgs)?;
            let f_branch = list.next().ok_or(EvalError::BadIfArgs)?;

            if list.next().is_some() {
                return Err(EvalError::BadIfArgs.into());
            }

            let cond = eval(cond, env.clone())?;

            if cond.truthy() {
                Ok(Trampoline::Continue(t_branch.clone(), env))
            } else {
                Ok(Trampoline::Continue(f_branch.clone(), env))
            }
        }
        Value::Form(Form::Lambda) => {
            let Value::Form(form) = pair.0 else {
                panic!("it's the pattern");
            };
            let (_, args, body) = make_callable(&form, &pair.1)?;

            let lambda = Value::Lambda {
                args,
                body,
                env: env.clone(),
            };

            Ok(Trampoline::Done(lambda))
        }
        _ => {
            let head = eval(&pair.0, env.clone())?;
            let is_macro = matches!(head, Value::Macro { .. });
            let tail = &pair.1;

            let (body, args, fenv) = match head {
                Value::Func {
                    name: _,
                    body,
                    args,
                }
                | Value::Macro {
                    name: _,
                    body,
                    args,
                } => (body, args, env.clone()),
                Value::Lambda { env, body, args } => (body, args, env.clone()),
                Value::Builtin(f, _) => {
                    let results = f.0(&tail
                        .to_cons_iter()
                        .map(|v| eval(v, env.clone()))
                        .collect::<Result<_, _>>()?)?;
                    return Ok(Trampoline::Done(results));
                }
                v => return Ok(Trampoline::Done(v.clone())),
            };

            let nenv = setup_env(tail, &args, is_macro, env.clone(), fenv)?;

            return if is_macro {
                let body = eval(&body, nenv)?;
                Ok(Trampoline::Continue(body, env.clone()))
            } else {
                // TODO(austin.jones): kill this clone
                Ok(Trampoline::Continue(body.deref().clone(), nenv.clone()))
            };
        }
    }
}

pub fn old_eval(ast: &Value, env: Rc<RefCell<Env>>) -> Result<Value> {
    let Value::Cons(pair) = ast else {
        return match ast {
            Value::Symbol(atom) => resolve(atom, env),
            _ => Ok(ast.clone()),
        };
    };

    match pair.0 {
        Value::Nil => Ok(Value::Nil),
        Value::Form(Form::Progn) => {
            let mut body = pair.1.to_cons_iter().peekable();

            while let Some(b) = body.next() {
                let val = eval(b, env.clone())?;

                if body.peek().is_none() {
                    return Ok(val);
                }
            }

            bail!("progn body can't be empty")
        }
        Value::Form(f) => eval_form(f, &pair.1, env),
        _ => {
            let head = eval(&pair.0, env.clone())?;
            let is_macro = matches!(head, Value::Macro { .. });
            let tail = &pair.1;

            let (body, args, fenv) = match head {
                Value::Func {
                    name: _,
                    body,
                    args,
                }
                | Value::Macro {
                    name: _,
                    body,
                    args,
                } => (body, args, env.clone()),
                Value::Lambda { env, body, args } => (body, args, env.clone()),
                Value::Builtin(f, _) => {
                    return f.0(&tail
                        .to_cons_iter()
                        .map(|v| eval(v, env.clone()))
                        .collect::<Result<_, _>>()?);
                }
                v => return Ok(v.clone()),
            };

            let nenv = setup_env(tail, &args, is_macro, env.clone(), fenv)?;

            let rv = eval(&body, nenv);

            if is_macro {
                eval(&rv?, env.clone())
            } else {
                rv
            }
        }
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

fn eval_form(form: Form, tail: &Value, env: Rc<RefCell<Env>>) -> Result<Value> {
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
            panic!("we shouldn't hit this");
        }
        Form::Eval => {
            let val = eval(tail, env.clone())?;
            eval(&val, env)
        }
        Form::DefineMacro | Form::Define => {
            define(&form, tail, env)?;

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

            let cond = eval(cond, env.clone())?;

            if cond.truthy() {
                eval(t_branch, env.clone())
            } else {
                eval(f_branch, env.clone())
            }
        }
        Form::Lambda => {
            let (_, args, body) = make_callable(&form, tail)?;

            let lambda = Value::Lambda {
                args,
                body,
                env: env.clone(),
            };

            Ok(lambda)
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
            eval(val, old_env.clone())?
        };

        new_env.borrow_mut().define(arg, val.clone());
    }

    if varidx < fargs.len() {
        let rest = if is_macro {
            citer.into_cons_list().clone()
        } else {
            citer
                .map(|val| eval(val, old_env.clone()))
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
            let value = eval(tail, env.clone())?;
            env.borrow_mut().define(atom, value);
        }
        Value::Cons(_) => {
            let (name, args, body) = make_callable(form, body)?;

            let Some(name) = name else {
                unreachable!();
            };
            let tag = name.clone();

            let proc = match form {
                Form::Define => Value::Func { name, args, body },
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
            Value::Symbol(a) => Ok((*a).to_owned()),
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

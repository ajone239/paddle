use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use anyhow::{Context, Ok, Result, bail};

use crate::cursor::process_file;
use crate::eval::{
    EvalError,
    env::Env,
    value::{Form, Value},
};
use crate::lexer::Span;

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
            let head = eval(pair.0.clone(), env.clone())
                .context(format!("error evaling head of list at {}", cons_span))?;
            let is_macro = matches!(head, Value::Macro { .. });
            let tail = &pair.1;

            let (body, args, fenv, fspan) = match head {
                Value::Func {
                    name: _,
                    body,
                    args,
                    env,
                    span,
                }
                | Value::Lambda {
                    env,
                    body,
                    args,
                    span,
                } => (body, args, env.clone(), span),
                Value::Macro {
                    name: _,
                    body,
                    args,
                    span,
                } => (body, args, env.clone(), span),
                Value::Builtin(f, name, _) => {
                    let results = f.0(&tail
                        .to_cons_iter()
                        .map(|v| eval(v.clone(), env.clone()))
                        .collect::<Result<_, _>>()?)
                    .context(format!("error calling [{}] at {}", name, cons_span))?;
                    return Ok(Trampoline::Done(results));
                }
                v => return Ok(Trampoline::Done(v.clone())),
            };

            let nenv = setup_env(tail, cons_span, &args, is_macro, env.clone(), fenv)?;

            let body = body.deref().clone().set_span(fspan);
            if is_macro {
                let body = eval(body, nenv)?;
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

fn quasi_quote_eval(ast: Value, env: Rc<RefCell<Env>>) -> Result<(Value, bool)> {
    match ast {
        Value::Cons(ref pair, span) => match pair.0 {
            // TODO(ajone239): this can cause a weird bug between quote and quasi quote
            Value::Nil(_) => Ok((ast.clone(), false)),
            Value::Form(Form::UnQuote, _) => {
                let val = eval(pair.1.clone(), env)?;
                Ok((val, false))
            }
            Value::Form(Form::UnQuoteSplicing, _) => {
                let val = eval(pair.1.clone(), env)?;
                Ok((val, true))
            }
            _ => {
                let (new_head, head_spliced) = quasi_quote_eval(pair.0.clone(), env.clone())?;
                let (new_tail, _) = quasi_quote_eval(pair.1.clone(), env.clone())?;

                if !head_spliced {
                    Ok((Value::Cons(Rc::new((new_head, new_tail)), span), false))
                } else {
                    Ok((new_head.splice(new_tail)?, false))
                }
            }
        },
        _ => Ok((ast.clone(), false)),
    }
}

fn eval_form(form: Form, span: Span, tail: Value, env: Rc<RefCell<Env>>) -> Result<Trampoline> {
    match form {
        Form::Quote => {
            let Value::Cons(tailtail, _) = tail else {
                unreachable!("this is how quote is formed: {}", tail)
            };
            Ok(Trampoline::Done(tailtail.0.clone()))
        }
        Form::QuasiQuote => {
            let Value::Cons(tailtail, _) = tail else {
                unreachable!("this is how quasiquote is formed")
            };
            let (qexpr, _) = quasi_quote_eval(tailtail.0.clone(), env)?;
            Ok(Trampoline::Done(qexpr))
        }
        Form::UnQuote => Err(EvalError::UnquoteOutsideQuasi(span).into()),
        // TODO(ajone239): update this error
        Form::UnQuoteSplicing => Err(EvalError::UnquoteOutsideQuasi(span).into()),
        Form::Require => {
            let mut list = tail.to_cons_iter();

            let file_name = list.next().ok_or(EvalError::BadRequireArgCount(0, span))?;

            let (file_name, file_span) = match file_name {
                Value::Str(atom, file_span) | Value::Symbol(atom, file_span) => (atom, file_span),
                _ => {
                    return Err(EvalError::BadRequireArgs(span).into());
                }
            };

            if list.next().is_some() {
                return Err(EvalError::BadRequireArgCount(2, span).into());
            }

            process_file(file_name.to_string().into(), env)
                .context(format!("error processing files at {}", file_span))?;

            Ok(Trampoline::Done(Value::NoPrint))
        }
        Form::Progn => {
            let mut body = tail.to_cons_iter().peekable();

            while let Some(b) = body.next() {
                if body.peek().is_none() {
                    return Ok(Trampoline::Continue(b.clone(), env.clone()));
                }
                let _ = eval(b.clone(), env.clone())?;
            }

            bail!("progn body can't be empty at {span}")
        }
        Form::Eval => {
            let val = eval(tail, env.clone())?;
            Ok(Trampoline::Continue(val, env.clone()))
        }
        Form::DefineMacro | Form::Define => {
            define(&form, span, &tail, env)?;

            Ok(Trampoline::Done(Value::NoPrint))
        }
        Form::SetBang => {
            let mut list = tail.to_cons_iter();
            let head = list.next().ok_or(EvalError::BadSetBangArgs(span))?;

            let tail = list.next().ok_or(EvalError::BadSetBangArgs(span))?;

            if list.next().is_some() {
                return Err(EvalError::BadSetBangArgs(span).into());
            }

            let Value::Symbol(atom, _) = head else {
                bail!("Bad set! head (at {}): {}", span, head)
            };

            let value = eval(tail.clone(), env.clone())?;

            env.borrow_mut()
                .set_bang(atom, value)
                .context(format!("error set!ing at {}", span))?;

            Ok(Trampoline::Done(Value::NoPrint))
        }
        Form::If => {
            let mut list = tail.to_cons_iter();

            let cond = list.next().ok_or(EvalError::BadIfArgs(span))?;
            let t_branch = list.next().ok_or(EvalError::BadIfArgs(span))?;
            let f_branch = list.next().ok_or(EvalError::BadIfArgs(span))?;

            if list.next().is_some() {
                return Err(EvalError::BadIfArgs(span).into());
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
                span,
            };

            Ok(Trampoline::Done(lambda))
        }
    }
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

fn define(form: &Form, span: Span, body: &Value, env: Rc<RefCell<Env>>) -> Result<()> {
    let mut list = body.to_cons_iter();
    let head = list.next().ok_or(EvalError::BadDefineArgs(span))?;

    match head {
        Value::Symbol(atom, _) => {
            let tail = list.next().ok_or(EvalError::BadDefineArgs(span))?;

            if list.next().is_some() {
                return Err(EvalError::BadDefineArgs(span).into());
            }
            let value = eval(tail.clone(), env.clone())?;
            env.borrow_mut().define(atom, value);
        }
        Value::Cons(_, _) => {
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
                    span,
                },
                Form::DefineMacro => Value::Macro {
                    name,
                    args,
                    body,
                    span,
                },
                _ => unreachable!("should only get here from define or definemacro"),
            };

            env.borrow_mut().define(tag.as_str(), proc);
        }
        _ => return Err(EvalError::BadDefineHead(span).into()),
    };

    Ok(())
}

type CallableInfo = (Option<String>, Vec<String>, Rc<Value>);
fn make_callable(form: &Form, body: &Value) -> Result<CallableInfo> {
    let bspan = body.get_span();
    let mut list = body.to_cons_iter();
    let head = list
        .next()
        .ok_or(EvalError::BadCallableArgs(*form, bspan))?;

    if !matches!(head, Value::Cons(_, _) | Value::Nil(_)) {
        return Err(EvalError::BadCallableArgs(*form, bspan).into());
    }

    let args_list = head
        .to_cons_iter()
        .map(|e| match e {
            Value::Symbol(a, _) => Ok(a.to_string()),
            _ => Err(EvalError::BadCallableArgsListType(*form, e.get_span()).into()),
        })
        .collect::<Result<Vec<String>, _>>()?;

    let (name, args) = if matches!(form, Form::Lambda) {
        (None, args_list)
    } else {
        if args_list.is_empty() {
            return Err(EvalError::BadCallableHead(*form, bspan).into());
        }
        (Some(args_list[0].to_owned()), args_list[1..].to_vec())
    };

    if list.is_empty() {
        return Err(EvalError::BadCallableBodyArgs(*form, bspan).into());
    }

    let body = Rc::new(Value::Cons(
        Rc::new((Value::Form(Form::Progn, bspan), list.into_cons_list())),
        bspan,
    ));

    Ok((name, args, body))
}

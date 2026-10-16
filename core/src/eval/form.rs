use std::cell::RefCell;
use std::rc::Rc;

use anyhow::{Context, Ok, Result, bail};

use crate::{
    cursor::process_file,
    env::Env,
    eval::{EvalError, Trampoline, eval, quasi::quasi_quote_eval},
    span::Span,
    value::{CallableInfo, Form, Value},
};

pub(super) fn eval_form(
    form: Form,
    span: Span,
    tail: Value,
    env: Rc<RefCell<Env>>,
) -> Result<Trampoline> {
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
            let last_file = env.borrow().current_file.to_owned();
            env.borrow_mut().current_file = file_name.to_string();

            process_file(file_name.to_string().into(), env.clone())
                .context(format!("error processing files at {}", file_span))?;

            env.borrow_mut().current_file = last_file;

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

            let info = CallableInfo {
                args,
                body,
                env: env.clone(),
            };

            let lambda = Value::Func(Rc::new(info), span);

            Ok(Trampoline::Done(lambda))
        }
        Form::WhoAmI => Ok(Trampoline::Done(Value::Str(
            env.borrow().current_file.to_owned().into(),
            Span::default(),
        ))),
    }
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

            let info = CallableInfo {
                args,
                body,
                env: env.clone(),
            };

            let proc = match form {
                Form::Define => Value::Func(Rc::new(info), span),
                Form::DefineMacro => Value::Macro(Rc::new(info), span),
                _ => unreachable!("should only get here from define or definemacro"),
            };

            env.borrow_mut().define(tag.as_str(), proc);
        }
        _ => return Err(EvalError::BadDefineHead(span).into()),
    };

    Ok(())
}

type MakeCallableInfo = (Option<String>, Vec<String>, Rc<Value>);
fn make_callable(form: &Form, body: &Value) -> Result<MakeCallableInfo> {
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

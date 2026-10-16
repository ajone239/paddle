use std::cell::RefCell;
use std::rc::Rc;

use anyhow::{Ok, Result};

use crate::{
    env::Env,
    value::{Form, Value},
};

use super::eval;

pub(super) fn quasi_quote_eval(ast: Value, env: Rc<RefCell<Env>>) -> Result<(Value, bool)> {
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

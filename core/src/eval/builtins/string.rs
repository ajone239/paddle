use std::{ops::Deref, rc::Rc};

use anyhow::{Context, Result, bail};

use crate::{eval::value::Value, lexer::Span};

// string-length
pub fn string_length(args: &Value) -> Result<Value> {
    let Value::Cons(pair, _) = args else {
        bail!("should give me a list");
    };

    if !matches!(pair.1, Value::Nil(_)) {
        bail!("only one arg");
    }

    match &pair.0 {
        Value::Str(s, _) | Value::Symbol(s, _) => Ok(Value::Num(s.len() as f64, Span::default())),
        _ => bail!("only strs for string-length"),
    }
}

// string-ref
pub fn string_ref(args: &Value) -> Result<Value> {
    let mut aiter = args.to_cons_iter();

    let sarg = aiter.next().context("must have 2 args")?;
    let iarg = aiter.next().context("must have 2 args")?;

    let Value::Num(idx, _) = iarg else {
        bail!("idx should be num");
    };

    let s = match sarg {
        Value::Str(s, _) | Value::Symbol(s, _) => s,
        _ => bail!("only strs for string-length"),
    };

    match s.bytes().nth(*idx as usize) {
        Some(b) => Ok(Value::Char(b, Span::default())),
        None => bail!("string-ref got {} with {} string", idx, s.len()),
    }
}

// substring
pub fn substring(args: &Value) -> Result<Value> {
    let mut aiter = args.to_cons_iter();

    let sarg = aiter.next().context("must have 2 args")?;
    let farg = aiter.next().context("must have 2 args")?;
    let targ = aiter.next();

    let Value::Num(from, _) = farg else {
        bail!("idx should be num");
    };
    let from = *from as usize;

    let s = match sarg {
        Value::Str(s, _) | Value::Symbol(s, _) => s,
        _ => bail!("only strs for string-length"),
    };

    let to = match targ {
        Some(Value::Num(v, _)) if *v as usize > s.len() => s.len(),
        Some(Value::Num(v, _)) => *v as usize,
        None => s.len(),
        _ => bail!("to must be a num"),
    };

    if from >= s.len() || from > to {
        bail!("invalid substring");
    }

    Ok(Value::Str(s[from..to].into(), Span::default()))
}

// string-append
pub fn string_append(args: &Value) -> Result<Value> {
    let s = args
        .to_cons_iter()
        .map(|v| match v {
            Value::Str(s, _) | Value::Symbol(s, _) => Ok(s.deref()),
            _ => bail!("only strs for string-append"),
        })
        .collect::<Result<String, _>>()?;

    Ok(Value::Str(s.into(), Span::default()))
}

// string->list
pub fn string_list(args: &Value) -> Result<Value> {
    let Value::Cons(pair, _) = args else {
        bail!("should give me a list");
    };

    if !matches!(pair.1, Value::Nil(_)) {
        bail!("only one arg");
    }

    match &pair.0 {
        Value::Str(s, _) | Value::Symbol(s, _) => Ok(Value::to_cons_list(
            s.bytes().map(|b| Value::Char(b, Span::default())).collect(),
        )),
        _ => bail!("only strs for string->list"),
    }
}

// string->num
pub fn string_num(args: &Value) -> Result<Value> {
    let Value::Cons(pair, _) = args else {
        bail!("should give me a list");
    };

    if !matches!(pair.1, Value::Nil(_)) {
        bail!("only one arg");
    }

    let s = match &pair.0 {
        Value::Str(s, _) | Value::Symbol(s, _) => s,
        _ => bail!("only strs for string->num"),
    };

    match s.parse() {
        Ok(val) => Ok(Value::Cons(
            Rc::new((
                Value::Symbol("ok".into(), Span::default()),
                Value::Num(val, Span::default()),
            )),
            Span::default(),
        )),
        Err(_) => Ok(Value::Cons(
            Rc::new((
                Value::Symbol("err".into(), Span::default()),
                Value::Str("badparse".into(), Span::default()),
            )),
            Span::default(),
        )),
    }
}

// list->string
pub fn list_string(args: &Value) -> Result<Value> {
    let Value::Cons(pair, _) = args else {
        bail!("should give me a list");
    };

    if !matches!(pair.1, Value::Nil(_)) {
        bail!("only one arg");
    }

    let s = pair
        .0
        .to_cons_iter()
        .map(|v| match v {
            Value::Char(b, _) => char::from(*b).to_string(),
            _ => v.to_string(),
        })
        .collect::<String>();

    Ok(Value::Str(s.into(), Span::default()))
}

pub fn make_char(args: &Value) -> Result<Value> {
    let Value::Cons(args, _) = args else {
        bail!("should give me an arg list");
    };

    if let Value::Cons(_, _) = &args.1 {
        bail!("only one arg");
    };

    let byte = match args.0 {
        Value::Symbol(ref args, _) | Value::Str(ref args, _) if args.len() == 1 => {
            args.bytes().next().unwrap()
        }
        Value::Num(byte, _) if (0.0..256.0).contains(&byte) => byte as u8,
        _ => bail!("char takes num, sym, or str"),
    };

    Ok(Value::Char(byte, Span::default()))
}

use crate::parser::Expr;

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Nil,
    Bool(bool),
    Num(f64),
    Str(&'a str),
    List(Vec<Value<'a>>),
}

pub fn eval<'a>(ast: &Expr<'a>) -> Value<'a> {
    match ast {
        Expr::Atom(atom, _) => resolve(atom),
        Expr::List(list, _) => apply(&list),
    }
}

fn apply<'a>(list: &[Expr<'a>]) -> Value<'a> {
    if list.is_empty() {
        return Value::Nil;
    }

    let func = list.first().unwrap();

    let args: Vec<_> = list.iter().skip(1).map(|e| eval(e)).collect();

    call(func, &args)
}

fn call<'a>(func: &Expr<'_>, args: &[Value<'a>]) -> Value<'a> {
    let Expr::Atom(s, _) = func else { panic!() };

    let mut args = args
        .iter()
        .filter(|a| matches!(a, Value::Num(_)))
        .map(|v| match v {
            Value::Num(n) => n,
            _ => todo!(),
        });

    let num = match *s {
        "+" => args.fold(0.0, |acc, x| acc + x),
        "*" => args.fold(1.0, |acc, x| acc * x),
        "-" => {
            let init = *args.next().unwrap();
            args.fold(init, |acc, x| acc - x)
        }
        "/" => {
            let init = *args.next().unwrap();
            args.fold(init, |acc, x| acc / x)
        }
        _ => panic!("operation not supported"),
    };
    Value::Num(num)
}

fn resolve<'a>(atom: &'a str) -> Value<'a> {
    if let Ok(num) = atom.parse::<f64>() {
        return Value::Num(num);
    }

    return Value::Str(atom);
}

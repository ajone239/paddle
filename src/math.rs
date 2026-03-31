use crate::eval::Value;

fn args_to_num<'a>(args: &'a [Value]) -> impl Iterator<Item = &'a f64> {
    args.iter().map(move |v| match v {
        Value::Num(n) => n,
        _ => todo!("bad call args: {:?}", args),
    })
}

pub fn add(args: &[Value]) -> Value {
    let args = args_to_num(args);
    let num = args.fold(0.0, |acc, x| acc + x);
    Value::Num(num)
}

pub fn min(args: &[Value]) -> Value {
    let mut args = args_to_num(args);
    let init = *args.next().unwrap();
    let num = args.fold(init, |acc, x| acc - x);
    Value::Num(num)
}

pub fn mul(args: &[Value]) -> Value {
    let args = args_to_num(args);
    let num = args.fold(1.0, |acc, x| acc * x);
    Value::Num(num)
}

pub fn div(args: &[Value]) -> Value {
    let mut args = args_to_num(args);
    let init = *args.next().unwrap();
    let num = args.fold(init, |acc, x| acc / x);
    Value::Num(num)
}

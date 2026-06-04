use super::*;

#[test]
fn stdlib_loads_without_error() {
    let env = Rc::new(RefCell::new(Env::default()));
    process(STD_LIB, env).expect("base.pd failed to load");
}

#[test]
fn fact_program_produces_ten_factorial() {
    // examples/fact.pd defines `fact` and calls `(fact 10)` → 3628800
    let env = Rc::new(RefCell::new(Env::default()));
    let mut results = process(FACT_PROGRAM, env).expect("fact.pd failed");
    assert_eq!(results.pop().unwrap(), num(3628800.0));
}

#[test]
fn import_program_produces_ten_factorial() {
    // examples/fact.pd defines `fact` and calls `(fact 10)` → 3628800
    let env = Rc::new(RefCell::new(Env::default()));
    let mut results = process(IMPORT_PROGRAM, env).expect("fact.pd failed");
    assert_eq!(results.pop().unwrap(), num(3628800.0));
}

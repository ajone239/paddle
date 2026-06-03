use super::*;

// null?

#[test]
fn null_of_nil() {
    assert_eq!(eval_str("(null? nil)"), Value::Bool(true));
}

#[test]
fn null_of_empty_list() {
    assert_eq!(eval_str("(null? '())"), Value::Bool(true));
}

#[test]
fn null_of_number() {
    assert_eq!(eval_str("(null? 1)"), Value::Bool(false));
}

#[test]
fn null_of_nonempty_list() {
    assert_eq!(eval_str("(null? '(1))"), Value::Bool(false));
}

#[test]
fn null_of_false() {
    assert_eq!(eval_str("(null? #f)"), Value::Bool(false));
}

// pair?

#[test]
fn pair_of_cons() {
    assert_eq!(eval_str("(pair? (cons 1 2))"), Value::Bool(true));
}

#[test]
fn pair_of_list() {
    assert_eq!(eval_str("(pair? '(1 2))"), Value::Bool(true));
}

#[test]
fn pair_of_nil() {
    assert_eq!(eval_str("(pair? nil)"), Value::Bool(false));
}

#[test]
fn pair_of_number() {
    assert_eq!(eval_str("(pair? 1)"), Value::Bool(false));
}

#[test]
fn pair_of_empty_list() {
    assert_eq!(eval_str("(pair? '())"), Value::Bool(false));
}

#[test]
fn pair_of_singleton_nil_list() {
    // '(nil) = Cons(Nil, Nil) — a proper one-element list, is a pair
    assert_eq!(eval_str("(pair? '(nil))"), Value::Bool(true));
}

#[test]
fn pair_of_nil_headed_list() {
    // '(nil nil) = Cons(Nil, Cons(Nil, Nil)) — is a pair
    assert_eq!(eval_str("(pair? '(nil nil))"), Value::Bool(true));
}

#[test]
fn pair_of_cons_nil_nil() {
    // (cons nil nil) same shape as '(nil)
    assert_eq!(eval_str("(pair? (cons nil nil))"), Value::Bool(true));
}

// number?

#[test]
fn number_of_integer() {
    assert_eq!(eval_str("(number? 1)"), Value::Bool(true));
}

#[test]
fn number_of_float() {
    assert_eq!(eval_str("(number? 3.14)"), Value::Bool(true));
}

#[test]
fn number_of_nil() {
    assert_eq!(eval_str("(number? nil)"), Value::Bool(false));
}

#[test]
fn number_of_bool() {
    assert_eq!(eval_str("(number? #t)"), Value::Bool(false));
}

#[test]
fn number_of_list() {
    assert_eq!(eval_str("(number? '(1))"), Value::Bool(false));
}

// atom?

#[test]
fn atom_of_number() {
    assert_eq!(eval_str("(atom? 1)"), Value::Bool(true));
}

#[test]
fn atom_of_nil() {
    assert_eq!(eval_str("(atom? nil)"), Value::Bool(true));
}

#[test]
fn atom_of_bool() {
    assert_eq!(eval_str("(atom? #t)"), Value::Bool(true));
}

#[test]
fn atom_of_symbol() {
    assert_eq!(eval_str("(atom? 'x)"), Value::Bool(true));
}

#[test]
fn atom_of_list() {
    assert_eq!(eval_str("(atom? '(1))"), Value::Bool(false));
}

#[test]
fn atom_of_cons() {
    assert_eq!(eval_str("(atom? (cons 1 2))"), Value::Bool(false));
}

#[test]
fn atom_of_singleton_nil_list() {
    // '(nil) is a Cons, not an atom
    assert_eq!(eval_str("(atom? '(nil))"), Value::Bool(false));
}

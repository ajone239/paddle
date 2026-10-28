use paddle_core::value::Form;

use super::*;

fn define_and_get_body(name: &str, expand: bool, definition: &str) -> Value {
    let env = Env::with_stdlib(expand).unwrap();
    process(definition, env.clone()).unwrap();

    let value = env.borrow().resolve(name).expect("define fell");

    let Value::Func(callable, _) = value else {
        panic!("shouldn't be anything else");
    };

    strip_value_span(&callable.body)
}

#[test]
fn regular_def() {
    let body = define_and_get_body("f", true, "(def (f x) (+ x 1))");

    assert_eq!(
        body,
        progn(cons(
            cons(sym("+"), cons(sym("x"), cons(num(1.0), nil()))),
            nil()
        ))
    );
}

#[test]
fn macro_expand_def() {
    let body = define_and_get_body("f", true, "(def (f x) (when x (+ x 1)))");

    assert_eq!(
        body,
        progn(cons(
            if_st(
                sym("x"),
                progn(cons(
                    cons(sym("+"), cons(sym("x"), cons(num(1.0), nil()))),
                    nil()
                )),
                quote(cons(nil(), nil()))
            ),
            nil()
        ))
    );
}

#[test]
fn macro_no_expand_def() {
    let body = define_and_get_body("f", false, "(def (f x) (when x (+ x 1)))");

    assert_eq!(
        body,
        progn(cons(
            cons(
                sym("when"),
                cons(
                    sym("x"),
                    cons(cons(sym("+"), cons(sym("x"), cons(num(1.0), nil()))), nil())
                )
            ),
            nil()
        ))
    );
}

#[test]
fn macro_expand_nested() {
    let body = define_and_get_body("f", true, "(def (f x y) (when x (unless y 1)))");

    assert_eq!(
        body,
        progn(cons(
            if_st(
                sym("x"),
                progn(cons(
                    if_st(
                        cons(sym("not"), cons(sym("y"), nil())),
                        progn(cons(num(1.0), nil())),
                        quote(cons(nil(), nil()))
                    ),
                    nil()
                )),
                quote(cons(nil(), nil()))
            ),
            nil()
        ))
    );
}

#[test]
fn macro_expand_recursive() {
    // `cond` recurses over its clauses while expanding, so the result is a
    // chain of ifs with no `cond` left in it.
    let body = define_and_get_body("f", true, "(def (f a b) (cond (a 1) (b 2)))");

    assert_eq!(
        body,
        progn(cons(
            if_st(sym("a"), num(1.0), if_st(sym("b"), num(2.0), nil())),
            nil()
        ))
    );
}

#[test]
fn macro_expand_deep_in_function() {
    let body = define_and_get_body(
        "f",
        true,
        "(def (f x) (if x (+ 1 (if x (* 2 (when x 3)) 0)) 0))",
    );

    let when_x_3 = if_st(
        sym("x"),
        progn(cons(num(3.0), nil())),
        quote(cons(nil(), nil())),
    );
    let times = cons(sym("*"), cons(num(2.0), cons(when_x_3, nil())));
    let inner_if = if_st(sym("x"), times, num(0.0));
    let plus = cons(sym("+"), cons(num(1.0), cons(inner_if, nil())));

    assert_eq!(body, progn(cons(if_st(sym("x"), plus, num(0.0)), nil())));
}

fn assert_expand_parity(definition: &str) {
    assert_eq!(
        define_and_get_body("f", true, definition),
        define_and_get_body("f", false, definition),
        "pre-expanded and non-expanded bodies differ for {definition}"
    );
}

#[test]
fn no_macro_body_parity_quoted_nil_termination() {
    assert_expand_parity("(def (f x) '())");
    assert_expand_parity("(def (f x) (if x 1 '()))");
    assert_expand_parity("(def (f x) (+ x 1) '())");
}

#[test]
fn no_macro_body_parity_false_termination() {
    assert_expand_parity("(def (f x) #f)");
    assert_expand_parity("(def (f x) (if x 1 #f))");
    assert_expand_parity("(def (f x) (+ x 1) #f)");
}

#[test]
fn macro_expand_recursive_or_into_let() {
    // `or` emits a `let` per operand, and `let` expands to a lambda application,
    // so the expander has to re-expand its own output.
    let body = define_and_get_body("f", true, "(def (f a b) (or a b))");

    assert_eq!(
        body,
        progn(cons(
            cons(
                cons(
                    form(Form::Lambda),
                    cons(
                        cons(sym("test"), nil()),
                        cons(
                            if_st(
                                sym("test"),
                                sym("test"),
                                cons(
                                    cons(
                                        form(Form::Lambda),
                                        cons(
                                            cons(sym("test"), nil()),
                                            cons(
                                                if_st(
                                                    sym("test"),
                                                    sym("test"),
                                                    Value::Bool(false, Span::default())
                                                ),
                                                nil()
                                            )
                                        )
                                    ),
                                    cons(sym("b"), nil())
                                )
                            ),
                            nil()
                        )
                    )
                ),
                cons(sym("a"), nil())
            ),
            nil()
        ))
    );
}

#[test]
fn macro_expand_three_levels_letn() {
    // letn -> letrec -> let -> lambda application
    let body = define_and_get_body(
        "f",
        true,
        "(def (f n) (letn loop ((i n)) (if i (loop i) i)))",
    );

    assert_eq!(
        body,
        progn(cons(
            cons(
                cons(
                    form(Form::Lambda),
                    cons(
                        cons(sym("loop"), nil()),
                        cons(
                            cons(
                                form(Form::SetBang),
                                cons(
                                    sym("loop"),
                                    cons(
                                        cons(
                                            form(Form::Lambda),
                                            cons(
                                                cons(sym("i"), nil()),
                                                cons(
                                                    if_st(
                                                        sym("i"),
                                                        cons(sym("loop"), cons(sym("i"), nil())),
                                                        sym("i")
                                                    ),
                                                    nil()
                                                )
                                            )
                                        ),
                                        nil()
                                    )
                                )
                            ),
                            cons(cons(sym("loop"), cons(sym("n"), nil())), nil())
                        )
                    )
                ),
                cons(nil(), nil())
            ),
            nil()
        ))
    );
}

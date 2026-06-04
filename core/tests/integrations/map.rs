use super::*;

#[test]
fn map_on_empty_list_returns_empty() {
    assert_eq!(run("(map (lambda (x) (* x 2)) '())"), Value::Nil);
}

#[test]
fn map_preserves_length() {
    assert_eq!(
        run("(length (map (lambda (x) (* x 2)) '(1 2 3)))"),
        num(3.0)
    );
}

#[test]
fn map_transforms_first_element() {
    assert_eq!(run("(car (map (lambda (x) (* x 2)) '(1 2 3)))"), num(2.0));
}

#[test]
fn map_works_with_named_function() {
    assert_eq!(
        run("(def (square x) (* x x)) (car (map square '(3 4 5)))"),
        num(9.0)
    );
}

#[test]
fn map_over_range() {
    // (map (lambda (x) (* x x)) (range 4)) has length 4
    assert_eq!(
        run("(length (map (lambda (x) (* x x)) (range 4)))"),
        num(4.0)
    );
}

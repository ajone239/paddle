use super::*;

#[test]
fn filter_on_empty_list_returns_empty() {
    assert_eq!(
        run("(filter (lambda (x) #t) '())"),
        Value::Nil(Span::default())
    );
}

#[test]
fn filter_remove_all() {
    assert_eq!(run("(length (filter (lambda (x) #f) '(1 2 3)))"), num(0.0));
}

#[test]
fn filter_keep_all() {
    assert_eq!(run("(length (filter (lambda (x) #t) '(1 2 3)))"), num(3.0));
}

#[test]
fn filter_count_passing_elements() {
    // keep x where x > 2: 3, 4, 5 → length 3
    assert_eq!(
        run("(length (filter (lambda (x) (< 2 x)) '(1 2 3 4 5)))"),
        num(3.0)
    );
}

#[test]
fn filter_first_passing_element() {
    // keep x where x > 2: first passing element is 3
    assert_eq!(
        run("(car (filter (lambda (x) (< 2 x)) '(1 2 3 4 5)))"),
        num(3.0)
    );
}

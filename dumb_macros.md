# Dumb Macros

## Intro

Meta-programming is cool--like really cool.
Writing a program to do something is cool; writing a program to write a program is _really cool_.
Things like the C preprocessor, Rust Macros, and C# Source generators allow for everything from cleaner/faster functions to framework like functionality.
But if you hang around in the meta-programming circles long enough, you will hear about Lisp macros.
You'll be thrust into Rackets syntax rule system.
However, in my experience, you won't get a good sense of why Lisp macros are any better than another system.

For a number of learning goals, I wrote a Lisp called Paddle (based on Racket (which I mistook for Racquet (hence Paddle))).
The pertinent goal was to learn why Lisp macros are so good.
Paddle has a full macro system with: delayed eval, quasiquote, unquote, unquote splicing, blah.
But these nice to haves can hide the power that vanilla Lisp is giving.
To hopefully explain why Lisp is lending to macros and why that's important, I have written some dumb macros.

## The problem

Let's start with the problem.
Lisps are semi-functional (functional in shape for sure (less functional in their side effect systems)).
So, everything is an expression, and all branches have to have a return statement.
You end up doing a lot of:

```racket
(define (foo bar)
    (if (null? bar)
        '()
        (baz bar)))
```

This is annoying after a bit.
It would be so nice to be able to just **macro** that pattern into use.
With the aforementioned macro system, in Paddle, you can simply:

```racket
(defmacro (unless cond body...)
 `(if ,cond
      (progn @body...)
      '()))

;usage
(define (foo bar)
    (unless (null? bar)
        (baz bar)))
```

But this doesn't look much different from:

```C
#define unless(cond, ...) \
    do { \
        if (!(cond)) { \
            __VA_ARGS__ \
        } \
    } while(0)

int foo(int bar) {
    unless(null == bar, baz(bar));
}
```

That's fair, where the cool part comes in is that we don't need that macro system at all.
We can just as well define the macro like this.

```racket
(define (unless cond body)
 (list
  'if (list 'not cond)
      body
      '()))

(define (foo bar)
    (eval (unless '(null? bar)
            '(baz bar))))
```

## A Lisp Primer Interlude

That last code snippet might have been a lot to look at if you aren't comfortable with Lisp.
So, the following is a small primer on what I think you need to know to get it given programming knowledge.

### `define`

The `=` operator as it is used in the C-likes is anathema to functional languages.
Functional languages much prefer "declaration" or "definition" as an idea.
There in you get `define`; it `define`s something in the current environment.

### `' | quote`

### `list`


### `eval`

## Homoiconicity

## Revisiting our dumb macro


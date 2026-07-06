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
    int rv = 0;
    unless(null == bar, rv = baz(bar));
    retrun rv
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

### Everything is a list

You may not see it looking at it, but everything in Lisp (besides atoms) are lists.

```racket
1 ; <- atom
x ; <- atom
'(1 2 3) ; <- list
(define x 1) ; <- list
(define (add1 x) (+ 1 x)) ; <- list
add1 ; <- atom
(add1 2) ; <- list

(define (foo bar)
    (unless (null? bar)
        (baz bar))) ; <- yep list
```

### `define`

The `=` operator as it is used in the C-likes is anathema to functional languages.
Functional languages much prefer "declaration" or "definition" as an idea.
There in you get `define`; it `define`s something in the current environment.
In paddle it has two forms: scalar and function define:

```racket
; scalar
(define x 1)
x ; => 1

; function
(define (add1 x) (+ 1 x))
(add1 2) ; => 3
```

**NOTE**: There's also a `defmacro` that doesn't what it says it does.

### `' | quote`

When a Lisp reads an expression, it wants to eval that thing pronto.
You can stop it doing this with `'`:

```racket
(+ 1 2) ; => 3
'(+ 1 2) ; => '(+ 1 2) ; still a list
```

### `list`

This function lets us make our precious lists.

```racket
(list 1 2) ; => '(1 2)
(list 'if #t 'here 'nothere) ; => '('if #t 'here 'nothere)
```

### `eval`

This essentially just says "evaluate the data I'm giving you".

```racket
'(+ 1 2) ; => '(+ 1 2) ; still a list
(eval '(+ 1 2)) ; => 3 ; we evaled!
```

## Homoiconicity

Homoiconicity is a $4 word that means "code is data and data is code".
Definitionally, it really means "same" "representation".
But practically, we've already covered what this means.

```racket
'(1 2 3) ; <- list
(define (add1 x) (+ 1 x)) ; <- list
```

One of the above is list of numbers to be `sum`ed or `map`ed.
The other is a function definition.
But they are both lists?
This is totally different than say Python:

```racket
[1, 2, 3] <- list
define add1(x):
    return x + 1 <- function block
```

So big whoop.
Wrong.
This makes our source code as easy to manipulate as data.
In a macro right?
Wrong again.
Even in a regular function.
Remember, code is data and data is code.

```racket
; car gets the head of the list
; cdr gets the tail
(define (get-func-name f)
    (car ; get func name
        (car ; grab function def head
            (cdr f)))); get rid of def

; note the `'`
(define func-def
    '(define (add1 x) (+ x 1)))

(eval func-def) ; execute the frozen definition
(get-func-name func-def) ; => add1
```

The sky is now the limit here.
See the examples.

## Revisiting our dumb macro

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

We now have the tools to understand that this is doing.
`unless` is a function that expects 2 lists `cond` and `body` that it can then massage into a new list.
This list is really source for a function.
It uses quoting and the `list` function to format the list into valid source.
With some imagination we see that the `foo` function expands to:

```racket
(define (foo bar)
    (eval '('if ('not '(null? bar))
            '(baz bar)
            '())))

; and the eval can be ran to give us essentially
(define (foo bar)
 (if (not (null? bar))
     (baz bar)
     ()))
```

And we don't even have any idea of a "macro" in the interpreter at this point.
We only have delayed execution and homoiconicity.

## More sophisticated work

However, an ergonomic if statement a happy programmer does not make.
Let's looks at another problem: you want to have chained ifs.
Let's turn 0-9 into their words.

``` racket
(define (dig-word num)
    (if (= num 1)
        'one
        (if (= num 2)
            'two
            (if (= num 3)
                'three
                ...))))
```

Nope, that's gonna be ugly.
So, let's define a function that will generate this nesting of `if`s for us.

```racket
(define else #t)
(define (cond pairs)
    (if (null? pairs)
      '()
      (list 'if (car (car pairs))
            (cadr (car pairs))
            (cond (cdr pairs)))))
```

For as long is `pairs` has something in it, this function will nest it into an `if`.
It is used thusly:

```racket
(define (dig-word num)
    (eval (cond '(
        ((= 1 num) 'one)
        ((= 2 num) 'two)
        ((= 3 num) 'three)
        ((= 4 num) 'four)
        ((= 5 num) 'five)
        ((= 6 num) 'six)
        ((= 7 num) 'seven)
        ((= 8 num) 'eight)
        ((= 9 num) 'nine)
        ((= 0 num) 'zero)
        (else 'too-big)))))
```

Again, with no idea of what a macro is, we are meta-programming.
This is just the beginning: macros, DSLs, frameworks, etc. can be wrought with these simple tools.

## Some sugar

But of course who would want too.
The manual quoting you see in the macros above is noisy.
The manual re-eval is fragile.
The manual list formatting is just annoying.
That's where the rest of the tools come in and you get something like:

```racket
(defm (unless cond body...)
 `(if (not ,cond) (progn @body...) '()))

(define else #t)
(defmacro (cond pairs...)
    (define (cond2 pairs)
      (when pairs
       `(if ,(car (car pairs))
            ,(cadr (car pairs))
            ,(cond2 (cdr pairs)))))
    (cond2 pairs...))
```

## Conclusion

Would you ever write macros like I just showed you? No.
But you can.
It is backed into the DNA of the language.
The C preprocessor is a whole separate step of compilation.
With Lisps, macros are more than first class citizens.
They are natural ways to make programs function.
This level of naturalness means that meta-programming is always on the tool belt and not just for the wizards.

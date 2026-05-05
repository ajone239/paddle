# Paddle

A Lisp interpreter written in Rust whose name is a pun that is built on my mistaking racket for racquet.
This is built as a learning project for exploring programming language implementation.

## Goal

Make a Lisp capable of running non-trivial programs, as a vehicle for learning
programming-language implementation and getting comfortable with Rust's memory
model.

## Roadmap

### Milestones

- [ ] M0 — writeups
    + writeup of the `dumb_macros.pd` experiment
    + full data flow from bytes to eval in the readme
- [ ] M1 — memory representation rework
    + [ ] cons cells (Rc-cells vs arena: deferred)
    + [ ] kill pervasive clones in `eval`/`apply`
    + [ ] fix nested-vector handling
- [ ] M2 — make the language runnable
    + [ ] tail-call optimization
    + [ ] variadic arguments (fix macros after this)
    + [ ] `set!`
    + [ ] `let` / `let*`
    + [ ] error/condition system usable from Paddle
    + [ ] string builtins
        * [ ] `string-length`
        * [ ] `string-ref`
        * [ ] `substring`
        * [ ] `string-append`
        * [ ] `string->list`
        * [ ] `list->string`
        * [ ] `string=?`
    + [ ] `getchar` builtin
    + [ ] `read-line` in Paddle
- [ ] M3 — goalpost programs
    + [ ] `paddle.pd` — meta-circular evaluator
    + [ ] `forth.pd` — Forth interpreter in Paddle
    + [ ] AoC days in Paddle

Out of scope: bytecode VM, lexer iterator, AST arena.

### Foundation (work so far)

#### Frontend
- [x] Lexer — tokenizes source into `LeftParen`, `RightParen`, `Quote`, `Symbol`
- [x] Source spans — line/column attached to every token
- [x] String literals — space-safe quoted strings
- [x] Escape sequences — `\"` and `\\` inside strings
- [x] Parser — recursive descent, produces `Expr::Atom` / `Expr::List`
- [x] Quote expansion — `'x` → `(quote x)` at parse time
- [x] Parse errors with source location

#### Evaluator
- [x] Value type design
- [x] Basic eval — literals, arithmetic, `quote`
- [x] Environment — `define`
- [x] Lambdas and closures
- [x] Macros — `define-macro`, quasiquote, unquote

#### Runtime
- [x] Standard library — arithmetic
- [x] Standard library — `car`, `cdr`, `cons`.
- [x] Standard library — `fold`, `map`, etc.
- [x] REPL
- [x] Line editing
- [x] Runtime errors with source location instead of panics
- [x] better printing
- [x] File runner — cursor-based multi-expression evaluation
- [x] :require
- [x] "(require xxx)"
- [x] comments

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
    + project layout
- [x] M1 — memory representation rework
    + [x] cons cells (Rc-cells)
    + [x] kill pervasive clones in `eval`/`apply`
    + [x] fix nested-vector handling
- [x] M2 — make the language runnable
    + [x] tail-call optimization
    + [x] variadic arguments (fix macros after this)
    + [x] `set!`
    + [x] `let` / `let*`
    + [x] `let <name>`
    + [x] error/condition system usable from Paddle
    + [x] string builtins
        * [x] `string-length`
        * [x] `string-ref`
        * [x] `substring`
        * [x] `string-append`
        * [x] `string->list`
        * [x] `string->num`
        * [x] `list->string`
        * [x] `string=?`
        * [x] `append`
    + [x] `getchar` builtin
    + [x] `read-line` in Paddle
- [ ] M3 — goalpost programs
    + [ ] `paddle.pd` — meta-circular evaluator
    + [ ] `forth.pd` — Forth interpreter in Paddle
    + [ ] AoC days in Paddle
- [ ] M4 -- onwards
    + [x] bust up the built-ins
    + [x] no print intrinsic
    + [ ] wasm playground
    + [ ] blend result macros and value.rs
    + [ ] `if __name__ == '__main__': <code>`
    + [ ] path based imports
    + [ ] fix env leak for functions

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

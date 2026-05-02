# Paddle

A Lisp interpreter written in Rust whose name is a pun that is built on my mistaking racket for racquet.
This is built as a learning project for exploring programming language implementation.

## Roadmap

### Frontend
- [x] Lexer — tokenizes source into `LeftParen`, `RightParen`, `Quote`, `Symbol`
- [x] Source spans — line/column attached to every token
- [x] String literals — space-safe quoted strings
- [x] Escape sequences — `\"` and `\\` inside strings
- [x] Parser — recursive descent, produces `Expr::Atom` / `Expr::List`
- [x] Quote expansion — `'x` → `(quote x)` at parse time
- [x] Parse errors with source location
- [ ] Lexer iterator
- [ ] Arena allocation ast

### Evaluator
- [x] Value type design
- [x] Basic eval — literals, arithmetic
- [x] `quote`
- [x] Environment — `define`
    + [x] variable lookup
    + [x] function lookup
- [x] Environment — scope
- [x] Lambdas and closures
- [-] dumb Macros
    + [x] when
    + [x] unless
    + [x] cond
    + [ ] loop
    + [ ] write about them
- [-] Macros —
    + [x] `define-macro`
        * [x] hook into define
        * [x] reggie functions
        * [x] make eval not eval args
    + [ ] quasiquote
    + [ ] unquote
    + [-] when
    + [-] unless
    + [-] cond
    + [ ] loop
- [ ] Tail call optimization
- [ ] kill clone being everywhere
- [ ] nested vectors being handled bad

### Runtime
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

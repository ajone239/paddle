# Paddle

A Lisp interpreter written in Rust whose name is a pun that is built on my mistaking racket for racquet.
This is built as a learning project for exploring programming language implementation.

# Data flow

The following is a rough flow for the interpreter.

- **Input**: Bytes from a file or stdin.
- **Lexer**: Chops those bytes into tokens.
- **Cursor**: Wraps the running of the parser, lower, and eval so multi-ast input can run.
- **Parser**: Grabs tokens in units of full AST.
    + **Parser Value**: A simplified data representation to keep the parser simple
    + **AST**: Abstract syntax tree (tokens with structure).
    + **Rest**: The remaining unused tokens.
- **Lower**: Converts the simple parser value to a rich eval value.
- **Evaluator**: Semi-recursively evaluates an AST reducing it to one value.
    + **Environment**: A persistent scratch pad for evaluation.
    + **Trampoline**: Bootstraps TCO into the evaluator.
- **Display**: Shows the results of the work to the user.
- **REPL**: The loop this all can run in.

<!--
:! graph-easy
-->
<!--
graph { flow: down }
[ user_input ] { border: 1px dotted black; }
[ source ] { border: 1px dotted black; }
[ loop ] { border: 1px dashed black; }
[ user_input ] -> [ lexer ]
[ source ] -> [ lexer ]
[ lexer ] -> [ cursor ]
[ cursor ] => [ parser ]
[ parser ] -> [ lower ]
[ parser ] -> [ rest ]
[ rest ] -> [ cursor ]
[ lower ] -> [ eval ]
[ eval ] -> [ display ]
[ eval ] -> [ trampoline ]
[ trampoline ] -> [ eval ]
[ trampoline ] <..> [ env ]
[ eval ] <-> [ env ]
[ display ] -> [ loop ]
-->

```
                   ...........
                   : source  :
                   :.........:
                     |
                     |
                     v
..............     +---------+
: user_input : --> |  lexer  |
:............:     +---------+
                     |
                     |
                     v
                   +---------+
  +--------------> | cursor  |
  |                +---------+
  |                  H
  |                  H
  |                  v
+------------+     +---------+
|    rest    | <-- | parser  |
+------------+     +---------+
                     |
                     |
                     v
                   +---------+
                   |  lower  |
                   +---------+
                     |
                     |
                     v
                   +----------------------------+
                   |            eval            | -+
                   +----------------------------+  |
                     |          ^    ^             |
                     |          |    |             |
                     |          |    |             |
                     v          |    v             |
                   +---------+  |  +------------+  |
                   | display |  |  |    env     |  |
                   +---------+  |  +------------+  |
                     |          |    ^             |
                     |          |    :             |
                     |          |    :             |
                     v          |    v             |
                   + - - - - +  |  +------------+  |
                   '  loop   '  +- | trampoline | <+
                   + - - - - +     +------------+
```

# Examples

- `base.pd`: Standard lib lisp functions.
- `macros.pd`: Standard lib lisp functions.
- `dumb_macros.pd`: A learning experiment for homoiconicity.
- `fact.pd`: Sanity integration test.
- `import.pd`: Import integration test.
- `y_combinator.pd`: Math fun.
- `forth.pd`: A forth interpreter.
- `paddle.pd`: A meta evaluator for my SICP badge.
- `nqueens.pd`: Solver for the N-Queens problem.
- `sudoku.pd`: Sudoku Solver.
- `wc.pd`: An implementation of the `wc` utility.

# TODO Some day

- wasm playground
- try out pest
- blend result macros and value.rs
- `if __name__ == '__main__': <code>`
- path based imports
- bytecode VM
- lexer iterator
- AST arena


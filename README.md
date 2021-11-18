# Oolisp
### Web-based LISP interpreter

```
Welcome to Oolisp, a Web-based LISP interpreter.

  Syntax & Types:
    number:
        - Numbers like we are all familiar with. (ie. 1, 1.1, 1.1e+13, 1.1e-13)
    symbol:
        - Symbols are names that can be assigned to any value. (ie. add, def, fun, some-var)
        usage: def {symbol-name} value
    string:
        - Strings are characters delimited by double quotes. (ie. "c'ect ci nest pa un pipe?", "hg king")
    s-expression:
        - S-Expressions are used to call and evaluate functions. (ie. (+ 1 2 3), (- (+ 9 1) (* 5 2)), (list 1 2 3 4), (== {} {}))
        usage: (function arg0 arg1 arg2)
    q-expression:
        - Q-Expressions are lists of values, remains unevaluated. (ie. {1 1 1 1}, {+ 9 (== {} {})})
        usage: {elem0 elem1 elem2}
    lambda:
        - Lambda functions are how you build functions, can be partially applied. (ie. ( {a b} {+ a b}))
        usage: ( {arg-list} {body})
```

Strongly influenced by:
- https://buildyourownlisp.com/
- https://mitpress.mit.edu/books/little-typer

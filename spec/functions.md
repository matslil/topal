# Function semantics

## Formal text

### TOPAL-FUNCTION-STATIC-NULLARY-001 — Static nullary function execution

A declaration `name is fn static () -> R` followed by one indented expression
shall introduce an immutable zero-parameter function named `name`. Applying
`name ()` shall evaluate its body in the declaration's lexical source context
and return that value exactly when it is classified by `R`. Declaration shall
not evaluate the body. Only bindings already visible at the declaration are
visible in its body; bindings declared later shall not be captured by calling
the function later. Each call shall evaluate the body once and shall expose
selection, entry, body decisions, and return in that order.

This initial executable function subset admits the explicit result classifiers
`Boolean`, `Int`, `Rational`, `String`, and `Unit`. Other headers, parameters,
multi-statement bodies, effects, and recursion remain unsupported until later
rules extend this subset.

### TOPAL-FUNCTION-STATIC-UNARY-001 — Static unary function execution

A declaration `name is fn static ( parameter : P ) -> R` followed by one
indented expression shall introduce an immutable one-parameter function. An
application `name argument` shall evaluate `argument` once in caller scope,
require its value to be classified by `P`, bind it to `parameter` only in a new
function scope containing the declaration's captured bindings, evaluate the
body once, and require the result to be classified by `R`.

Argument evaluation and validation shall precede function selection and entry.
Test traces shall expose parameter binding, selection, entry, body decisions,
and return in that semantic order. The supported parameter and result
classifiers are the same initial value classifiers admitted by
`TOPAL-FUNCTION-STATIC-NULLARY-001`.

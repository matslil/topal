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

### TOPAL-FUNCTION-STATIC-BINARY-001 — Static binary infix function execution

A declaration `name is fn static ( left : L, right : R ) -> O` shall introduce
a two-operand function. An infix application `left-value name right-value`
shall evaluate the left and right operands once in source order, validate them
against `L` and `R`, and bind them to `left` and `right` in declaration order
within the function scope defined by
`TOPAL-FUNCTION-STATIC-UNARY-001`.

Parameter names shall be distinct. Argument validation shall complete before
function selection and entry; a failed shape, arity, or classifier check shall
not enter the function. Test traces shall expose each successful binding in
declaration order before selection, entry, body decisions, and return.

### TOPAL-FUNCTION-BLOCK-001 — Function block execution

An executable function body may contain one or more equally indented statements.
They shall execute from top to bottom in a fresh function scope according to
the block and sequencing rules of the language design. Each non-final statement
shall either introduce a binding, explicitly discard its value, or evaluate to
`Unit`; the final statement's value is the function result and shall satisfy the
declared result classifier.

Bindings introduced by the block shall become visible only to later statements
in the same invocation and shall not escape it. Test traces and debugger history
shall expose their creation and subsequent resolution between function entry
and return.

### TOPAL-FUNCTION-RETURN-001 — Explicit function return

A statement `return expression` shall evaluate `expression` once and complete
the nearest enclosing function immediately with that value. Statements after
the return in the same function invocation shall not be evaluated. The returned
value shall satisfy the function's declared result classifier through the same
validation used for an implicit final-expression result.

`return` outside a function body shall be rejected. Test traces and debugger
history shall expose the explicit-return decision before the common function
return event and shall contain no decisions from skipped statements.

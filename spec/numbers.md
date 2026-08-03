# Numeric semantics

## Formal text

### TOPAL-NUM-INT-001 — Exact integer domain

`Int` contains every mathematical integer and the two distinguished endpoints
`-Infinity` and `+Infinity`. Finite `Int` values have arbitrary precision: no
accepted operation may overflow, wrap, saturate, truncate, or select a
machine-width result unless that behavior belongs to a separately named type or
explicit operation. This realizes `TOPAL-REQ-SAFE-001` and
`TOPAL-REQ-DETERMINISM-001`.

### TOPAL-NUM-LITERAL-001 — Integer literal construction

For a syntactically valid unsigned integer lexeme `d` of radix `r`, evaluation
constructs the unique finite `Int` value
`sum(i, digit(d_i) * r^(n-i-1))`, ignoring valid grouping underscores. For an
adjacent signed lexeme `-d`, evaluation constructs the additive inverse of that
value. In particular, `-0` has the ordinary integer value zero; directional
evidence required by an expected classifier is separate from `Int` identity.

### TOPAL-NUM-SYMBOL-001 — Fixed numeric callable names

The root language scope reserves the callable names `+`, `-`, `*`, `/`, and
`^`. They participate in ordinary prefix or binary application and overload
selection; they do not introduce a distinct precedence grammar or permit
arbitrary symbolic declarations. This realizes `TOPAL-REQ-MODEL-001` and
`TOPAL-REQ-TOOLS-001`.

### TOPAL-NUM-ADD-001 — Finite exact integer addition

For finite `a : Int` and `b : Int`, the root `+` overload is a total, pure
function `(Int, Int) -> Int` and evaluates to the unique mathematical sum of
`a` and `b`. Its result is finite, carries no arithmetic error, and is
independent of operand representation or evaluation tool. This realizes
`TOPAL-REQ-SAFE-001`, `TOPAL-REQ-DETERMINISM-001`, and
`TOPAL-REQ-INTEROP-001`.

### TOPAL-NUM-NEG-001 — Finite exact integer negation

For finite `a : Int`, prefix `-` selects a total, pure root overload
`Int -> Int` and evaluates to the unique additive inverse of `a`. Its result is
finite and cannot overflow. Adjacent `-a` numeric source is instead literal
construction under `TOPAL-NUM-LITERAL-001`; both forms produce the same `Int`
value but take distinct syntactic and semantic paths.

### TOPAL-NUM-SUB-001 — Finite exact integer subtraction

For finite `a : Int` and `b : Int`, binary `-` selects a total, pure root
overload `(Int, Int) -> Int` and evaluates to the unique mathematical difference
of `a` and `b`. Its result is finite, carries no arithmetic error, and is
independent of operand representation or evaluation tool. This realizes
`TOPAL-REQ-SAFE-001`, `TOPAL-REQ-DETERMINISM-001`, and
`TOPAL-REQ-INTEROP-001`.

### TOPAL-NUM-MUL-001 — Finite exact integer multiplication

For finite `a : Int` and `b : Int`, binary `*` selects a total, pure root
overload `(Int, Int) -> Int` and evaluates to the unique mathematical product
of `a` and `b`. Its result is finite, carries no arithmetic error, and cannot
overflow. Multiplication has no syntactic precedence over addition or
subtraction; mixed applications group left-to-right under
`TOPAL-SYN-GRAMMAR-001`. This realizes `TOPAL-REQ-SAFE-001`,
`TOPAL-REQ-DETERMINISM-001`, and `TOPAL-REQ-INTEROP-001`.

Other numeric domains and the remaining fixed callable names are outside this
initial formal numeric subset until later rules define their applicable
overloads. Their tokens remain reserved, and a conforming partial tool shall
reject unsupported applications explicitly rather than infer behavior.

## Explanatory notes

The initial subset establishes arithmetic paths incrementally before
formalizing exact division, exponentiation, or infinite operands. Left
association is fixed by `TOPAL-SYN-GRAMMAR-001`; for example, `2 + 3 * 4`
produces `20`, while `2 + ( 3 * 4 )` produces `14`.

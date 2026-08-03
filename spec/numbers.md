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

Other numeric domains and the remaining fixed callable names are outside this
initial formal numeric subset until later rules define their applicable
overloads. Their tokens remain reserved, and a conforming partial tool shall
reject unsupported applications explicitly rather than infer behavior.

## Explanatory notes

The initial subset deliberately establishes one complete arithmetic path before
formalizing subtraction, multiplication, exact division, exponentiation, or
infinite operands. Left association is fixed by `TOPAL-SYN-GRAMMAR-001`; for
example, `1 + 2 + 3` selects the finite `Int` addition overload twice.

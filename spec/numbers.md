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

### TOPAL-NUM-NAT-001 — Nonnegative integer refinement

`Nat` classifies exactly the finite `Int` values greater than or equal to zero
in the currently implemented finite numeric subset. It preserves the underlying
exact integer value and introduces no unsigned representation, truncation, or
wrapping. A function parameter or result classified as `Nat` accepts zero and
positive `Int` values and rejects negative values at the applicable validation
boundary.

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

### TOPAL-NUM-RATIONAL-001 — Canonical exact rational values

A finite `Rational` value is one mathematical ratio represented canonically as
coprime integers `(n, d)` with `d > 0`. Construction moves any sign to `n`,
divides both components by their greatest common divisor, and represents zero
as `(0, 1)`. Canonical identity and equality depend on the mathematical ratio,
not the source spelling or construction path.

### TOPAL-NUM-RATIONAL-LITERAL-001 — Exact rational literals

A syntactically valid fractional decimal or base-ten exponent literal constructs
the unique canonical finite `Rational` equal to its decimal expansion. An
adjacent minus negates that exact value. Grouping underscores have no numeric
effect, trailing fractional zeroes do not change identity, and no binary
approximation or machine-width exponent is introduced.

### TOPAL-NUM-RAT-NEG-001 — Finite exact rational negation

For finite `a : Rational`, prefix `-` selects the total, pure root overload
`Rational -> Rational` and returns the canonical additive inverse of `a`.

### TOPAL-NUM-RAT-ADD-001 — Finite exact rational addition

For finite `a, b : Rational`, binary `+` selects the total, pure root overload
`(Rational, Rational) -> Rational` and returns their canonical mathematical sum.

### TOPAL-NUM-RAT-SUB-001 — Finite exact rational subtraction

For finite `a, b : Rational`, binary `-` selects the total, pure root overload
`(Rational, Rational) -> Rational` and returns their canonical mathematical
difference.

### TOPAL-NUM-RAT-MUL-001 — Finite exact rational multiplication

For finite `a, b : Rational`, binary `*` selects the total, pure root overload
`(Rational, Rational) -> Rational` and returns their canonical mathematical
product.

### TOPAL-NUM-RAT-DIV-001 — Finite exact rational division

For finite `a : Rational` and finite nonzero `b : Rational`, binary `/` selects
the total, pure root overload `(Rational, Rational) -> Rational` and returns
their canonical exact quotient. A statically zero divisor is rejected under
`TOPAL-NUM-DIVZERO-001`. These finite rational arithmetic rules realize
`TOPAL-REQ-SAFE-001`, `TOPAL-REQ-DETERMINISM-001`, and
`TOPAL-REQ-INTEROP-001`.

### TOPAL-NUM-INT-RATIONAL-CONVERT-001 — Canonical exact embedding

The canonical lossless conversion from finite `n : Int` to `Rational` produces
the value `(n, 1)`. A mixed finite exact application may use this conversion
once per `Int` operand to match an existing rational overload under
`TOPAL-TYPE-CONVERT-001`; it does not create a separate mixed-domain overload
and conversion quality does not alter source-order selection.

The reverse conversion is implicit only when retained static evidence proves
that the canonical rational denominator is one. Otherwise conversion to `Int`
is validation and returns the applicable typed `Result`. No implicit conversion
chains are introduced. This realizes `TOPAL-REQ-MODEL-001`,
`TOPAL-REQ-SAFE-001`, and `TOPAL-REQ-INTEROP-001`.

### TOPAL-NUM-DIV-001 — Finite exact integer division

For finite `a : Int` and finite nonzero `b : Int`, binary `/` selects a total,
pure root overload `(Int, Int) -> Rational` and evaluates to the canonical exact
ratio `(a, b)` under `TOPAL-NUM-RATIONAL-001`. It never truncates, rounds, or
implicitly returns the operand type, including when the denominator reduces to
one. This realizes `TOPAL-REQ-SAFE-001`, `TOPAL-REQ-DETERMINISM-001`, and
`TOPAL-REQ-INTEROP-001`.

### TOPAL-NUM-DIVZERO-001 — Statically evident zero division

An exact division whose divisor is statically proven zero is rejected with the
`division-by-zero` arithmetic diagnostic before evaluation. A partial tool may
conservatively reject division when it cannot establish the nonzero obligation;
it shall not assume nonzero or produce an undefined value. Later dynamic-input
rules may instead construct the typed `Result` required by the error model.

### TOPAL-NUM-DYNAMIC-DIVZERO-001 — Dynamic Rational zero division

Within a function explicitly returning
`Result ( Rational, lang arithmetic ArithmeticErrorCode )`, division of a
`Rational` by a dynamically obtained zero constructs an `Error` with code
`division-by-zero`. Its domain is derived from the qualified reporting overload
`root./(Rational,Rational)` and its source provenance identifies the divisor
occurrence. The error returns through the declared Result path; a statically
evident zero remains a diagnostic under `TOPAL-NUM-DIVZERO-001`.

### TOPAL-NUM-ARITHMETIC-ERROR-001 — Arithmetic error-code vocabulary

The qualified namespace `lang arithmetic` publishes the nominal enum type
`ArithmeticErrorCode` with alternatives `out-of-range`, `not-representable`,
`division-by-zero`, and `indeterminate`. These code identities are independent
of the compiler-derived reporting provenance stored in `Error.domain`.

### TOPAL-NUM-POW-001 — Finite natural integer exponentiation

For finite `a : Int` and finite `e : Nat`, binary `^` selects a total, pure root
overload `(Int, Nat) -> Int` and evaluates to exact repeated multiplication:
`a ^ 0 = 1` and `a ^ (e + 1) = (a ^ e) * a`. Consequently `0 ^ 0 = 1` as the
empty product. The result is finite and cannot overflow. A finite negative
`Int` exponent does not satisfy this overload; rational negative-exponent
semantics require a later overload. Exponentiation has no hidden precedence and
groups under `TOPAL-SYN-GRAMMAR-001`. This realizes `TOPAL-REQ-SAFE-001`,
`TOPAL-REQ-DETERMINISM-001`, and `TOPAL-REQ-INTEROP-001`.

### TOPAL-NUM-RAT-POW-001 — Finite natural Rational exponentiation

For finite `a : Rational` and finite `e : Nat`, binary `^` selects the total,
pure root overload `(Rational, Nat) -> Rational` and evaluates by exact repeated
multiplication: `a ^ 0 = Rational (1, 1)` and
`a ^ (e + 1) = (a ^ e) * a`. Consequently a zero Rational base raised to zero
is the Rational multiplicative identity. The canonical result cannot overflow.

A negative `Int` exponent does not satisfy this natural-exponent overload and
instead follows `TOPAL-NUM-RAT-NEG-POW-001`. No `Int`-to-`Rational` conversion
applies to the exponent; test traces shall identify `root.^(Rational,Nat)`
directly for nonnegative exponents.

### TOPAL-NUM-RAT-NEG-POW-001 — Exact negative Rational exponentiation

For finite nonzero `a : Rational` and finite negative `e : Int`, binary `^`
selects `(Rational, Int) -> Rational` and returns the exact reciprocal of
`a ^ absolute(e)`. No rounding or overflow occurs. A statically evident zero
base is rejected as division by zero; a dynamic zero base requires the
arithmetic Result failure path and constructs `division-by-zero` with reporting
domain `root.^(Rational,Int)` plus separate source provenance.

### TOPAL-NUM-COMPARE-001 — Finite exact total ordering

Finite `Int` and finite `Rational` each provide `TotalOrder` using their exact
mathematical order. A same-domain comparison produces exactly `Less`, `Equal`,
or `Greater`; the predicates `<`, `>`, `<=`, and `>=` select the corresponding
result or result set and return `Boolean`. Mixed finite `Int` and `Rational`
comparison first applies `TOPAL-NUM-INT-RATIONAL-CONVERT-001` to the integer and
then uses rational order. These predicates use ordinary left-to-right
application and have no special chaining rule.
This is the numeric realization of `TOPAL-TYPE-ORDERING-001`.

Other numeric domains and the remaining fixed callable names are outside this
initial formal numeric subset until later rules define their applicable
overloads. Their tokens remain reserved, and a conforming partial tool shall
reject unsupported applications explicitly rather than infer behavior.

## Explanatory notes

The initial subset establishes arithmetic paths incrementally before
formalizing remaining dynamic arithmetic failures or infinite operands. Left
association is fixed by `TOPAL-SYN-GRAMMAR-001`; for example, `2 + 3 * 4`
produces `20`, while `2 + ( 3 * 4 )` produces `14`.

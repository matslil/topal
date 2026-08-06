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
The named application `negate a` selects the corresponding root operation and
returns the same additive inverse while remaining separately traceable.

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

### TOPAL-NUM-RATIONAL-CONSTRUCT-001 — Closed finite Rational construction

Prefix application `Rational (numerator, denominator)` with two statically
closed finite `Int` components and a nonzero denominator constructs the
canonical value required by `TOPAL-NUM-RATIONAL-001`. A statically zero
denominator is diagnosed under `TOPAL-NUM-DIVZERO-001`. Dynamic component
validation and its structured zero-denominator failure are outside this rule.

### TOPAL-NUM-RATIONAL-CONSTRUCT-DYNAMIC-001 — Dynamic Rational construction

Prefix application `Rational (numerator, denominator)` with dynamically
obtained finite `Int` components returns
`Result ( Rational, lang arithmetic ArithmeticErrorCode )`. A nonzero
denominator produces the canonical finite value under
`TOPAL-NUM-RATIONAL-001`. A directionless zero denominator constructs
`division-by-zero` when the numerator is nonzero and `indeterminate` when it is
zero. Both failures use reporting domain `root.Rational(Int,Int)` and source
provenance at the constructor product. Statically evident failures are source
diagnostics. This constructor never produces infinity without separate
directional-zero evidence.

### TOPAL-NUM-RATIONAL-LITERAL-001 — Exact rational literals

A syntactically valid fractional decimal or base-ten exponent literal constructs
the unique canonical finite `Rational` equal to its decimal expansion. An
adjacent minus negates that exact value. Grouping underscores have no numeric
effect, trailing fractional zeroes do not change identity, and no binary
approximation or machine-width exponent is introduced.

### TOPAL-NUM-RAT-NEG-001 — Finite exact rational negation

For finite `a : Rational`, prefix `-` selects the total, pure root overload
`Rational -> Rational` and returns the canonical additive inverse of `a`.
The named `negate a` operation returns the same canonical value through an
ordinary named-application path.

### TOPAL-NUM-ABS-001 — Finite exact absolute value

For finite `a : Int` or `a : Rational`, `absolute a` selects the corresponding
total, pure root overload and returns `a` when it is nonnegative or its exact
additive inverse otherwise. The result retains the operand's numeric domain,
does not convert or round, and cannot overflow.

### TOPAL-NUM-ZERO-001 — Exact numeric zero construction

`zero Int`, `zero Nat`, and `zero Rational` shall select total, pure type-directed root
operations and construct the unique additive identity of the named exact
numeric domain. The Int and Nat results are exact integer zero; the Rational result is
canonical rational zero. Construction shall not infer a domain from context or
convert an already constructed value.

### TOPAL-NUM-ONE-001 — Exact numeric one construction

`one Int`, `one Nat`, and `one Rational` shall select total, pure type-directed root
operations and construct the unique multiplicative identity of the named exact
numeric domain. The Int and Nat results are exact integer one; the Rational result is
canonical rational one. Construction shall not infer a domain from context or
convert an already constructed value.

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

Explicit prefix construction `Rational n` for `n : Int` exposes the same
canonical embedding as an ordinary expression and produces `Rational (n, 1)`.
It is total, exact, and does not construct a Result.

### TOPAL-NUM-DIV-001 — Finite exact integer division

For finite `a : Int` and finite nonzero `b : Int`, binary `/` selects a total,
pure root overload `(Int, Int) -> Rational` and evaluates to the canonical exact
ratio `(a, b)` under `TOPAL-NUM-RATIONAL-001`. It never truncates, rounds, or
implicitly returns the operand type, including when the denominator reduces to
one. This realizes `TOPAL-REQ-SAFE-001`, `TOPAL-REQ-DETERMINISM-001`, and
`TOPAL-REQ-INTEROP-001`.

### TOPAL-NUM-RATIONAL-INT-EXACT-001 — Closed exact Rational narrowing

When a statically closed exact expression produces canonical `Rational (n, 1)`
and its immediate classified binding requires `Int`, the value shall satisfy
that context as exact integer `n`. The conversion is lossless and traceable; it
does not round or truncate. A closed Rational with any other denominator is a
source diagnostic at the initializer. Dynamic narrowing requires separate
validation and failure semantics and is not implied by this rule.

### TOPAL-NUM-RATIONAL-INT-VALIDATE-001 — Dynamic exact Rational validation

When an `Int`-classified binding in a function returning
`Result ( Int, lang arithmetic ArithmeticErrorCode )` receives a dynamically
obtained finite `Rational`, it shall validate the canonical denominator. A
denominator of one produces the exact numerator as `Int`; any other denominator
constructs `not-representable` with reporting domain `root.Int(Rational)` and
source provenance at the binding initializer. The failed Result then propagates
under `TOPAL-TYPE-RESULT-PROJECT-001`. Validation never rounds or truncates.

### TOPAL-NUM-INT-CONSTRUCT-001 — Exact checked Int construction

Prefix application `Int value` performs exact checked construction. An `Int`
operand is preserved. A finite `Rational (n, 1)` produces `n : Int`. A closed
Rational with another denominator is a source diagnostic; a dynamically
obtained one constructs `not-representable` with reporting domain
`root.Int(Rational)`. Construction never rounds or truncates and exposes its
success or structured failure in the test trace.

### TOPAL-NUM-NAT-CONSTRUCT-001 — Checked Nat constraint construction

Prefix application `Nat value` validates an `Int` against the nonnegative Nat
constraint under `TOPAL-TYPE-CONSTRAINT-001`. A nonnegative operand is preserved
with reusable Nat evidence. A closed negative operand is a source diagnostic; a
dynamically obtained negative operand constructs `out-of-range` with reporting
domain `root.Nat(Int)`. Construction does not clamp, wrap, or otherwise replace
the exact integer value.

### TOPAL-NUM-DIVZERO-001 — Statically evident zero division

An exact division whose divisor is statically proven zero is rejected with the
`division-by-zero` arithmetic diagnostic before evaluation. A partial tool may
conservatively reject division when it cannot establish the nonzero obligation;
it shall not assume nonzero or produce an undefined value. Later dynamic-input
rules may instead construct the typed `Result` required by the error model.

### TOPAL-NUM-INT-MODULO-001 — Euclidean integer modulo

For finite `a : Int` and finite nonzero `b : Int`, binary `%` returns the unique
Euclidean remainder `r : Int` satisfying `a = b*q + r` for some integer `q` and
`0 <= r < absolute b`. The sign of either operand does not change this range.
A statically evident zero divisor is rejected under `TOPAL-NUM-DIVZERO-001`.
A dynamically obtained zero constructs `division-by-zero` with reporting domain
`root.%(Int,Int)` and separate divisor source provenance within a compatible
arithmetic Result contract.

### TOPAL-NUM-INT-QUOTIENT-MODULO-001 — Euclidean integer quotient and modulo

For finite `a : Int` and finite nonzero `b : Int`, binary `/%` returns the
product `(q, r) : (Int, Int)` uniquely satisfying `a = b*q + r` and
`0 <= r < absolute b`. Its remainder equals `a % b` under
`TOPAL-NUM-INT-MODULO-001`. Literal and dynamic zero divisors follow the same
diagnostic and structured Error rules, with dynamic reporting domain
`root./%(Int,Int)`.

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

### TOPAL-NUM-THREE-WAY-COMPARE-001 — Exact three-way comparison

For finite Int and Rational operands, `<=>` selects their applicable exact
`TotalOrder` and returns the nominal `Comparison` alternative `Less`, `Equal`,
or `Greater`. Same-domain comparison uses mathematical order. Mixed Int and
Rational comparison first applies the canonical lossless Int-to-Rational
conversion. The operator evaluates each operand once and does not create a
Boolean or apply chaining semantics.

Other numeric domains and the remaining fixed callable names are outside this
initial formal numeric subset until later rules define their applicable
overloads. Their tokens remain reserved, and a conforming partial tool shall
reject unsupported applications explicitly rather than infer behavior.

## Explanatory notes

The initial subset establishes arithmetic paths incrementally before
formalizing remaining dynamic arithmetic failures or infinite operands. Left
association is fixed by `TOPAL-SYN-GRAMMAR-001`; for example, `2 + 3 * 4`
produces `20`, while `2 + ( 3 * 4 )` produces `14`.

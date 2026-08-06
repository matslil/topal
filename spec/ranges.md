# Range semantics

### TOPAL-RANGE-INCLUSIVE-001 — Inclusive Int range construction

For finite `lower : Int` and `upper : Int`, binary `lower .. upper` constructs
the closed `Range Int` predicate accepting exactly values `v` for which
`lower <= v` and `v <= upper`. When `upper < lower`, the range is empty rather
than erroneous. Construction does not imply enumeration, progression, or a
successor operation. The application groups under `TOPAL-SYN-GRAMMAR-001`.

### TOPAL-RANGE-MEMBERSHIP-001 — Int range membership

For `value : Int` and `interval : Range Int`, both `value in interval` and
`interval contains value` return the same Boolean predicate result. A closed
range accepts the value exactly when `lower <= value` and `value <= upper`.
Consequently every value is rejected by a reversed, empty range. Each operand
is evaluated once.

### TOPAL-RANGE-RATIONAL-001 — Rational range construction and membership

Finite Rational endpoints construct a closed `Range Rational` with the same
inclusive and empty-range semantics. Mixed Int and Rational endpoints first use
the single canonical `Int`-to-`Rational` conversion. Membership accepts Rational
values and canonically embedded Int values, compares exactly, and never rounds.

### TOPAL-RANGE-CLASSIFIER-001 — Range domain classification

An inclusive range constructed from Int endpoints satisfies `Range Int`. A
range constructed from Rational endpoints, including mixed endpoints after
canonical conversion, satisfies `Range Rational`. These classifiers are valid
for bindings, function parameters, and function results; the range retains its
ordered endpoint domain across an ordinary call.

### TOPAL-RANGE-INTERSECTION-001 — Range conjunction

For two ranges over the same supported ordered endpoint domain, `left and right`
constructs their predicate intersection. Its lower bound is the greater lower
bound and its upper bound is the lesser upper bound. Disjoint inputs therefore
produce a reversed empty range rather than an error. Each operand is evaluated
once, and the result retains the shared Range classifier.

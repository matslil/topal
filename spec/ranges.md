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

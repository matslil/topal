# Range semantics

### TOPAL-RANGE-BOUNDS-001 — Int range construction

For finite `lower : Int` and `upper : Int`, the four binary range operators
construct these `Range Int` predicates:

| operator | accepted values |
| --- | --- |
| `lower .. upper` | `lower <= v` and `v < upper` |
| `lower <.. upper` | `lower < v` and `v < upper` |
| `lower ..= upper` | `lower <= v` and `v <= upper` |
| `lower <..= upper` | `lower < v` and `v <= upper` |

When no value satisfies the selected comparisons, the range is empty rather
than erroneous. In particular, equal endpoints form a singleton only for
`..=`. Construction does not imply enumeration, progression, or a successor
operation. Each application groups under `TOPAL-SYN-GRAMMAR-001`.

### TOPAL-RANGE-MEMBERSHIP-001 — Int range membership

For `value : Int` and `interval : Range Int`, both `value in interval` and
`interval contains value` return the same Boolean predicate result. Membership
uses the endpoint comparisons selected under `TOPAL-RANGE-BOUNDS-001`.
Consequently every value is rejected by an empty range. Each operand is
evaluated once.

### TOPAL-RANGE-RATIONAL-001 — Rational range construction and membership

Finite Rational endpoints support all four operators with the same endpoint
and empty-range semantics. Mixed Int and Rational endpoints first use
the single canonical `Int`-to-`Rational` conversion. Membership accepts Rational
values and canonically embedded Int values, compares exactly, and never rounds.

### TOPAL-RANGE-CLASSIFIER-001 — Range domain classification

Any range constructed from Int endpoints satisfies `Range Int`. A
range constructed from Rational endpoints, including mixed endpoints after
canonical conversion, satisfies `Range Rational`. These classifiers are valid
for bindings, function parameters, and function results; the range retains its
ordered endpoint domain across an ordinary call.

### TOPAL-RANGE-INTERSECTION-001 — Range conjunction

For two ranges over the same supported ordered endpoint domain, `left and right`
constructs their predicate intersection. Its lower bound is the greater lower
bound and its upper bound is the lesser upper bound. When equal bounds have
different inclusivity, the intersection excludes that bound. Disjoint inputs
therefore produce an empty range rather than an error. Each operand is evaluated
once, and the result retains the shared Range classifier.

### TOPAL-RANGE-EMPTY-001 — Empty range observation

`empty? interval` SHALL return true exactly when the interval's lower bound is
greater than its upper bound, or the bounds are equal and either bound is
excluded. It SHALL return false otherwise. Observation SHALL retain the range
and endpoint domain unchanged and SHALL NOT enumerate members.

### TOPAL-RANGE-BOUND-001 — Bound observation

`range-lower interval` and `range-upper interval` SHALL return the retained
lower and upper endpoint respectively, preserving the exact endpoint
classifier. `range-lower-inclusive? interval` and
`range-upper-inclusive? interval` SHALL report whether the respective endpoint
is included. These operations apply only where the implemented range form has
both finite explicit bounds; they SHALL NOT invent sentinels for absent bounds.

### TOPAL-RANGE-VALUE-SELECTION-001 — Convex value selection

`collection select range` shall treat Range as its ordinary value predicate,
retain exactly matching entries, and preserve source order and multiplicity.
The visible result shall retain the source collection kind; retained
`SelectionOf source range` evidence shall not expose a storage representation.

### TOPAL-RANGE-INDEX-SELECTION-001 — Convex index selection

`sequence select-index range` shall retain entries whose zero-based indexes
belong to the Range Int under its selected endpoint rules. List results remain Lists; String positions
are user-perceived Character indexes and results remain String. The operation
may retain `RangeSelectionOf source range` and `SliceOf source` evidence, but
these facts shall not change equality or expose whether storage is shared.

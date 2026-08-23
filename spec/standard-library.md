# Standard-library conformance

### TOPAL-LIB-SOURCE-001 — Authoritative declarations

The standard library SHALL be defined by ordinary version-selected Topal source
declarations. Every source tool SHALL resolve a published declaration through
the same package-relative module path and SHALL NOT maintain a separate native
definition of its observable semantics.

All fundamental-library declarations SHALL be published directly in the flat
`std` namespace. Internal source categorization SHALL NOT add intermediate
components to their public names. Algorithm libraries beyond these fundamental
operations SHALL use separate namespaces rather than extending `std`.

### TOPAL-LIB-DEPENDENCY-001 — Explicit standard-library selection

Application and test source SHALL acquire the `std` namespace only by declaring
`use library std ( version is V )` as specified by `TOPAL-SYN-LIBRARY-001`.
Tools SHALL NOT inject `std` into source contexts that omit the declaration.
The source declaration identifies a dependency, while tool configuration,
package metadata, or a lockfile determines its location and exact resolved
artifact.

### TOPAL-LIB-DIFFERENTIAL-001 — Cross-tool observations

For each executable standard-library example, conforming interpreter,
debugger, and compiler executions SHALL agree on the resulting value or
diagnostic, the applicable semantic decisions, and every fundamental debugging
trace event. Tool-purpose testing events MAY add evidence but SHALL NOT replace
or duplicate a fundamental debugging event.

### TOPAL-LIB-VERSION-001 — Version isolation

A standard-library package SHALL be checked using the language and Unicode
revisions selected by its source contexts. Cached checked-source or GEIR
artifacts with a different source-package key SHALL be rejected rather than
silently reinterpreted. An unsupported source revision SHALL produce a version
diagnostic before any declaration from that context becomes visible.

### TOPAL-LIB-SUBSTITUTION-001 — Optional compiler substitution

A compiler MAY replace a standard-library declaration only after matching its
exact structural identity and required capability evidence. The replacement
SHALL be observationally equivalent to executing the authoritative Topal
source. Absence of a substitution SHALL NOT affect program correctness.

### TOPAL-LIB-ORDERING-001 — Generic scalar extrema

The `std` namespace SHALL provide `min`, `max`, and `min-max` for
two values of one exact type that supplies `TotalOrder`. `min-max` SHALL return
the lesser value followed by the greater value. All three functions SHALL
select their left argument when the arguments compare equal.

### TOPAL-LIB-OPTIONAL-001 — Generic Optional composition

The `std` namespace SHALL provide Optional presence queries, payload
mapping and chaining, predicate filtering, value and Optional fallback,
pairwise zipping, and nested-Optional flattening. Each operation SHALL preserve
the exact related payload classifiers established by its inputs and any
higher-order result. The module SHALL NOT provide forced extraction.

### TOPAL-LIB-RESULT-001 — Generic Result and Error composition

The `std` namespace SHALL provide Result success and failure queries,
success mapping and chaining, explicit Error mapping and recovery, value and
Result fallback, pairwise zipping, and nested-Result flattening. Operations
which do not explicitly transform an Error SHALL preserve its complete domain,
code, detail, cause, and source provenance. Related success classifiers and
error-code vocabularies SHALL remain exact through every operation.

### TOPAL-LIB-EXACT-NUMERIC-001 — Derived exact-number operations

The `std` namespace SHALL provide exact-number sign and distance in their strongest
closed exact domains, Euclidean `gcd`, parity and divisibility predicates, and
fallible rational reciprocal. Partial arithmetic SHALL retain the arithmetic
Result vocabulary and compiler-derived reporting domain. No operation in this
module may round, overflow, or introduce parsing or presentation policy. Exact
`sum` and `product` folds SHALL preserve `Int` or `Rational` according to their
List entry classifier, use the corresponding additive or multiplicative
identity for an empty List, and evaluate in source order.

### TOPAL-LIB-RANGE-001 — Convex range utilities

The `std` namespace SHALL provide range bound observation, bound pairing,
intersection, overlap testing, convex hull, and discrete Int adjacency. Generic
operations SHALL retain one exact `TotalOrder` endpoint classifier. They SHALL
preserve convex-predicate semantics and SHALL NOT imply enumeration.

### TOPAL-LIB-TEXT-001 — Fundamental Unicode text utilities

The `std` namespace SHALL provide canonical normalization and equality,
default caseless equality, exact prefix/suffix/containment queries, Unicode
whitespace trimming, exact replacement, and repetition. Every operation SHALL
use the selected language context's pinned Unicode policy and SHALL remain
separate from parsing, formatting, locale policy, and encoded bytes.

### TOPAL-LIB-FINITE-001 — Derived finite fold algorithms

The `std` namespace SHALL provide generic existential, universal, and
negative quantifiers, predicate counting, and first-match search over finite
Lists. Implementations SHALL use ordinary finite folds, preserve the exact
element classifier, visit entries in source order, and return Optional rather
than force extraction when no match exists.

`filter-map` SHALL retain every present transformed payload in source order and
discard absent transformation results. `flat-map` SHALL concatenate transformed
Lists in source order while preserving each transformed List's internal order.
Both operations SHALL retain exact related input and output classifiers and
require finite input Lists.

### TOPAL-LIB-GENERATOR-001 — Lazy generator constructors

The `std` namespace SHALL provide integer enumeration as a replay-free
lazy generator. It SHALL yield only when resumed, retain its distinct final
return, and preserve the language generator-close protocol when abandoned. It
SHALL NOT materialize an unbounded traversal or imply that a linear generator
can be replayed. Generic repetition is deferred until generator declarations can
bind classifier parameters without weakening their linear continuation type.

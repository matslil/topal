# Standard-library conformance

### TOPAL-LIB-SOURCE-001 — Authoritative declarations

The standard library SHALL be defined by ordinary version-selected Topal source
declarations. Every source tool SHALL resolve a published declaration through
the same package-relative module path and SHALL NOT maintain a separate native
definition of its observable semantics.

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

The fundamental ordering module SHALL provide `min`, `max`, and `min-max` for
two values of one exact type that supplies `TotalOrder`. `min-max` SHALL return
the lesser value followed by the greater value. All three functions SHALL
select their left argument when the arguments compare equal.

### TOPAL-LIB-OPTIONAL-001 — Generic Optional composition

The fundamental Optional module SHALL provide presence queries, payload
mapping and chaining, predicate filtering, value and Optional fallback,
pairwise zipping, and nested-Optional flattening. Each operation SHALL preserve
the exact related payload classifiers established by its inputs and any
higher-order result. The module SHALL NOT provide forced extraction.

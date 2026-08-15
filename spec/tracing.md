# Semantic tracing

### TOPAL-TRACE-PROFILE-001 — Additive profiles

A trace envelope SHALL contain an ordered canonical collection named
`profiles`. `debugging` and `testing` are independent initial profiles.
Selecting `testing` SHALL add test-decision evidence to debugging events and
SHALL NOT duplicate an event already supplied by `debugging`. Encoding format
and adapter behavior SHALL NOT change event identity or meaning.

### TOPAL-TRACE-FUNDAMENTAL-001 — Debugging fundamentals

The fundamental debugging event vocabulary SHALL contain `create`, `destroy`,
and `access` cases for semantic values and `entry` and `exit` cases for
functions. Operators SHALL be functions for this rule. These events SHALL
describe semantic execution independently of allocation, representation,
copy, move, borrow, or sharing choices.

Binding an alias, function failure, resource operation, message transfer, task
scheduling, and transaction lifecycle SHALL NOT add another fundamental event
kind. A testing profile MAY add binding and decision evidence. Higher-level
events SHALL be derived through `TOPAL-INTRO-TRACE-001`.

### TOPAL-TRACE-IDENTITY-001 — Typed identity and provenance

Every event SHALL identify its typed event group and case through structured
Topal identity. A derived event SHALL retain the identity of its observer and
input provenance. Function events SHALL use the resolved structured function
identity, including for compiler-owned functions such as `lang task switch-to`.

### TOPAL-TRACE-ADAPTER-001 — Stream adapters

Adapters MAY encode, store, aggregate, or interact with the typed stream,
including through native serialization, CTF, Google Trace Event, a source
debugger, or a native debugger bridge. A live debugger adapter MAY synchronize
at a selected event and control execution. Adapter behavior SHALL remain
outside application semantics.

# Composite layout and information-flow semantics

## Formal text

### TOPAL-LAYOUT-SHAPE-001 — Semantic shape

`Shape` SHALL be an ordered product of unique dimension identities and exact
`Nat` extents. Shape is semantic static evidence and SHALL remain distinct from
storage representation. Zero extent SHALL be represented exactly and SHALL NOT
permit an out-of-range index.

### TOPAL-LAYOUT-BLOCK-001 — Multidimensional address calculation

A multidimensional layout SHALL name every shape dimension exactly once in its
order, use positive block extents, and provide compatible complete strides when
explicit. Checked address calculation SHALL reject overflow, overlap,
misalignment, and indexes outside the shape. Boundary tiles SHALL retain their
actual extents and padding SHALL be unobservable.

### TOPAL-LAYOUT-COMPONENT-001 — Component organization

`Interleaved`, `Separated`, and `Blocked N` SHALL recursively map every semantic
component path without changing the semantic value. The checker SHALL derive
one total storage size and SHALL reject missing, duplicate, or overlapping
component coverage.

### TOPAL-LAYOUT-SPARSE-001 — Canonical sparse representation

A sparse layout SHALL record value and index layouts, compressed dimensions,
block shape, canonical order, duplicate policy, and zero policy. Construction
SHALL validate bounds and block shape and canonicalize or reject duplicates
according to policy. Reading SHALL produce the same semantic array independent
of representation.

### TOPAL-LAYOUT-VIEW-001 — Checked views and conversions

A zero-copy view SHALL require proof that one storage satisfies both layouts
and that lifetimes and access rights agree. Otherwise conversion SHALL be an
explicit fallible operation carrying allocation, transfer, and peak-live
evidence. A foreign layout description SHALL grant no ABI, symbol, pointer, or
unchecked address authority.

### TOPAL-INFO-LATTICE-001 — Verified information policy

`InformationPolicy` SHALL provide confidentiality and integrity label sets,
flow, join, and meet. The checker SHALL verify reflexivity, transitivity,
antisymmetry, and least-upper/greatest-lower laws over the declared decidable
domain before accepting labels. Names alone SHALL NOT establish those laws.

### TOPAL-INFO-IMPLICIT-001 — Explicit and control-flow propagation

Direct information flow SHALL join the source label into the destination.
Control dependent values, errors, effects, and messages SHALL additionally
join the current program-counter label. A boundary SHALL reject a label not
permitted by its exact policy relation.

### TOPAL-INFO-DECLASSIFY-001 — Scoped confidentiality authority

`declassify A V` SHALL lower confidentiality only when unforgeable `A` names the
same policy, source, target, purpose, and live scope as `V`. It SHALL record an
auditable `Declassification A` effect and preserve provenance. Source,
generated code, and foreign adapters SHALL NOT construct, copy, or widen `A`.

### TOPAL-INFO-ENDORSE-001 — Scoped integrity authority

`endorse A V` SHALL raise integrity only under the analogous exact unforgeable
authority and SHALL record `Endorsement A`. Declassification SHALL NOT imply
endorsement or vice versa. Static labels MAY erase only after all flow and
authority checks succeed.

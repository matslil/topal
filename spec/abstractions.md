# Generic abstraction semantics

## Formal text

### TOPAL-ABSTRACTION-PATTERN-001 — Simultaneous type-pattern instantiation

A generic type pattern shall bind each declared parameter exactly once and
replace all occurrences simultaneously with the selected exact type identity.
Instantiation shall fail when a required parameter has no argument and shall
not infer an argument by erasing nominal identity, constraints, or structure.

### TOPAL-ABSTRACTION-EVIDENCE-001 — Retained generic evidence

Every accepted generic instantiation shall retain the originating declaration,
the canonical parameter-to-type substitution, and the exact instantiated
result. Ordered overload selection may inspect this evidence but shall not
rewrite it after selection.

### TOPAL-CAPABILITY-EVIDENCE-001 — Canonical capability evidence

Capability evidence shall identify the atomic capability, exact classified
subject, and ordinary declaration identity assigned to every promised operation
role. Selecting a role shall invoke that recorded declaration rather than
restart unqualified overload resolution.

### TOPAL-CAPABILITY-COHERENCE-001 — Evidence coherence

For one capability and exact subject, a static context shall contain at most one
canonical role assignment. Repeating identical evidence is idempotent;
conflicting assignments are an ambiguity error before execution.

### TOPAL-CAPABILITY-COMPOSE-001 — Capability conjunction and alternatives

`A and B` over capability expressions shall require the same implicit subject
and retain every promise from both operands. `A or B` shall retain distinct
evidence alternatives without claiming either operand unconditionally.
Composition is canonical and idempotent and does not create a runtime method
namespace.

### TOPAL-INTERFACE-SHAPE-001 — Implementation-independent call shapes

An interface shall retain its nominal declaration identity and a uniquely named
set of function or generator operation shapes. A function shape contains its
classified inputs and result; a generator shape additionally contains its
yielded and resumed classifiers. Shapes do not select implementation locations.

### TOPAL-INTERFACE-IMPLEMENTATION-001 — Intentional complete implementation

An interface implementation shall explicitly identify the interface and supply
exactly one ordinary declaration identity for each operation role, with no
missing or additional roles. Matching declarations outside that construction
do not establish conformance. Packaging retains both the interface identity and
the selected declaration identities.

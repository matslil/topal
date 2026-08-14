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

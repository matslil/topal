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

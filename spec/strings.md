# String semantics

## Formal text

### TOPAL-STRING-CONCAT-001 — Plain exact concatenation

For plain `String` values `a` and `b`, `a concatenate b` shall produce the
plain `String` whose preserved Unicode scalar sequence is the complete sequence
of `a` followed by the complete sequence of `b`. It shall not normalize,
reinterpret escapes, interpolate placeholders, or insert a separator.

Constraint-aware concatenation which retains shared normalization evidence is
applicable only when that evidence is implemented and selected. Its absence
shall not change the exact plain-string result defined here.

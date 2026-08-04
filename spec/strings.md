# String semantics

## Formal text

### TOPAL-STRING-EMPTY-001 — Empty construction

`empty String` shall construct the unique plain `String` whose preserved
Unicode scalar sequence has length zero. Construction shall not consult locale,
normalization, encoding, or external state.

### TOPAL-STRING-CONCAT-001 — Plain exact concatenation

For plain `String` values `a` and `b`, `a concat b` shall produce the
plain `String` whose preserved Unicode scalar sequence is the complete sequence
of `a` followed by the complete sequence of `b`. It shall not normalize,
reinterpret escapes, interpolate placeholders, or insert a separator.

Constraint-aware concatenation which retains shared normalization evidence is
applicable only when that evidence is implemented and selected. Its absence
shall not change the exact plain-string result defined here.

### TOPAL-STRING-LITERAL-COMPOSE-001 — Adjacent literal composition

Two or more adjacent string literal primaries in one expression shall compose
at construction into one plain `String` containing their preserved Unicode
scalar sequences in source order. This rule applies only while every composed
operand is a literal. A binding, function result, or other string-valued
expression requires the explicit `concat` operation.

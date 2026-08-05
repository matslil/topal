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

### TOPAL-STRING-CHARACTER-COUNT-001 — User-perceived character count

For a plain `String` value `text`, `character-count text` shall produce the
finite nonnegative `Int` equal to the number of extended grapheme clusters in
the preserved Unicode sequence under the language context's pinned Unicode
segmentation data. Empty text has count zero. Canonically equivalent sequences
shall be segmented as preserved rather than normalized before counting.

### TOPAL-STRING-ENTRY-COUNT-001 — String sequence entry count

Because plain `String` provides `Sequence Character`, `entry-count text` shall
produce exactly the same finite nonnegative `Int` as `character-count text`
under `TOPAL-STRING-CHARACTER-COUNT-001`. It shall count user-perceived
characters, not Unicode scalar values, encoded bytes, or display columns.

### TOPAL-STRING-UTF8-BYTE-COUNT-001 — Prospective UTF-8 byte count

For a plain `String` value `text`, `text byte-count Utf8` shall produce the
finite nonnegative `Int` equal to the number of bytes in the standard UTF-8
encoding of its preserved Unicode scalar sequence. The operation observes a
prospective encoding boundary; it shall not attach an encoding to `text`,
normalize it, or count user-perceived characters or display columns.

### TOPAL-STRING-EMPTY-PREDICATE-001 — String emptiness

For a plain `String` value `text`, `empty? text` shall produce `true` exactly
when its preserved Unicode scalar sequence is empty and `false` otherwise. It
shall agree with both `character-count text = 0` and `entry-count text = 0`
without normalizing, encoding, or otherwise transforming the value.

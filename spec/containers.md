# Container semantics

### TOPAL-TYPE-LIST-CONSTRUCT-001 — Homogeneous List construction

In an expected `List T` context, `Empty` shall construct the empty List and
`Entry (value, remaining-list)` shall prepend `value` to a recursively
constructed List. Every entry shall satisfy `T`. A mismatched entry or a
remainder that is not a List shall be rejected at its source span.

### TOPAL-DECISION-LIST-001 — Total List decomposition

A complete List decision shall contain both `Empty then action` and
`Entry (first, rest) then action`, unless `otherwise` supplies the missing
alternative. The Entry alternative shall bind the first value as `T` and the
remaining entries as `List T` only in its selected action.

### TOPAL-TYPE-LIST-EQUALITY-001 — Structural List equality

Two `List T` values shall be equal exactly when they contain the same number of
entries and corresponding entries are equal in order under equality for `T`.

### TOPAL-LIST-PREPEND-001 — Front insertion

`list prepend value` shall produce `Entry (value, list)` and require `value` to
satisfy the List's element classifier.

### TOPAL-LIST-APPEND-001 — Back insertion

`list append value` shall preserve every existing entry in order and place the
new conforming value last.

### TOPAL-LIST-CONCAT-001 — Ordered List concatenation

`left concat right` shall require equal element classifiers and produce every
left entry followed by every right entry without removing duplicates.

### TOPAL-LIST-ENTRY-COUNT-001 — List cardinality

`entry-count list` shall return the nonnegative number of entries in the List.

### TOPAL-LIST-EMPTY-PREDICATE-001 — List emptiness

`empty? list` shall be true exactly when the List has no entries.

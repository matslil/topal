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

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

### TOPAL-LIST-EMPTY-001 — Explicit empty List construction

`empty List T` shall construct an empty `List T` without requiring an enclosing
expected classifier.

### TOPAL-LIST-ONE-001 — Singleton List construction

`one value` shall construct a one-entry List whose element classifier is the
structural classifier of `value`. Numeric calls such as `one Int` retain their
existing numeric overload.

### TOPAL-LIST-UNCONS-001 — Total front decomposition

`uncons list` shall return `None (T, List T)` for an empty `List T`. For a
nonempty List it shall return `Some (first, rest)`, where `first` is the first
entry and `rest` is a `List T` containing every later entry in order.

### TOPAL-LIST-FIRST-001 — Total first projection

`first list` shall return `None T` for an empty `List T` and `Some value` for a
nonempty List, where `value` is its first entry.

### TOPAL-LIST-REST-001 — Total remaining-List projection

`rest list` shall return `None (List T)` for an empty `List T` and `Some tail`
for a nonempty List, where `tail` contains every entry after the first in order.

### TOPAL-TYPE-LIST-RECURSIVE-001 — Recursive List classifiers

`List T` shall accept recursively supported element classifiers, including
products and another `List U`. Construction, function boundaries, equality,
and projections shall preserve every nested classifier and entry order.

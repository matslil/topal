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

### TOPAL-LIST-STABLE-SORT-001 — Stable exact-numeric List ordering

`values stable-sort` and `values stable-sort-descending` SHALL accept finite
`List Int` and `List Rational` values and return the same entries in ascending
or descending ordinary numeric order respectively. Entries that compare equal
SHALL retain their relative source order. The input List SHALL remain unchanged.
Other element classifiers SHALL be rejected until an explicit comparison
policy is supported.

### TOPAL-LIST-SEQUENCE-ALGORITHMS-001 — Derived sequence mechanisms

Finite Lists SHALL support first and last equal-value index search, modular
left and right rotation, positive-size chunking and sliding windows,
zero-based enumeration, adjacent equal-run grouping, and shortest-length zip.
All transformations SHALL retain source order except for the stated rotation,
preserve exact nested classifiers, and leave their inputs unchanged. Index
search SHALL return `None Nat` when absent. Zero chunk or window size SHALL be
rejected; an oversized window SHALL produce an empty outer List.

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

### TOPAL-LIST-CONTAINS-ENTRY-001 — List entry containment

`list contains-entry value` shall be true exactly when an equal entry occurs.

### TOPAL-LIST-CONTAINS-SEQUENCE-001 — Consecutive List containment

`list contains-sequence pattern` shall be true exactly when every pattern entry
occurs consecutively and in order. The empty pattern is contained.

### TOPAL-LIST-CONTAINS-SUBSEQUENCE-001 — Ordered List subsequence containment

`list contains-subsequence pattern` shall preserve pattern order while allowing
gaps between matched entries. The empty pattern is contained. All three
containment operations require equality for the element classifier.

### TOPAL-LIST-REVERSE-001 — Immutable List reversal

`list reverse` shall produce a List with the same element classifier, entry
count, and multiplicities in exactly the opposite order.

### TOPAL-LIST-REMOVE-FIRST-001 — Remove the first equal entry

`list remove-first value` shall remove only the earliest entry equal to `value`.
If none is equal, it shall preserve the List unchanged.

### TOPAL-LIST-REMOVE-ALL-001 — Remove every equal entry

`list remove-all value` shall remove every entry equal to `value`. Both removal
operations shall preserve the relative order of retained entries, preserve the
element classifier, and require equality for that classifier.

### TOPAL-COLLECTION-MAP-001 — Ordered List transformation

`list map transformation` shall call the contextual unary transformation once
for each entry in List order and return the transformed results in that order.

### TOPAL-COLLECTION-SELECT-001 — Ordered List selection

`list select predicate` shall call the contextual unary Boolean predicate once
for each entry in List order and retain exactly the entries for which it returns
true, without changing their relative order.

### TOPAL-COLLECTION-FOLD-001 — Left ordered List fold

`list fold initial step` shall call the contextual binary step in List order,
passing the preceding state and current entry. The empty List result is
`initial`; otherwise the final step result is returned.

### TOPAL-LIST-BOUNDARY-CHECK-001 — Checked List positions

List boundaries range from zero through the entry count; List indexes range
from zero through one less than the entry count. A statically invalid position
shall be diagnosed. An unchecked invalid runtime Nat shall return `out-of-range`
from the lexical operation domain rather than clamp or partially modify a List.

### TOPAL-LIST-INSERT-AT-001 — Boundary insertion

`list insert-at boundary inserted` shall insert either one conforming value or
every entry of a same-classifier List before the entry at `boundary`, preserving
the relative order of both existing and inserted entries.

### TOPAL-LIST-SPLIT-AT-001 — Boundary split

`list split-at boundary` shall return a product containing the prefix before
the boundary and the suffix beginning there, preserving every entry exactly.

### TOPAL-LIST-TAKE-001 — Checked prefix

`list take count` shall return the first `count` entries and use the same
validity rule as `split-at`.

### TOPAL-LIST-DROP-001 — Checked suffix

`list drop count` shall return every entry after the first `count` entries and
use the same validity rule as `split-at`.

### TOPAL-LIST-REMOVE-INDEX-001 — Checked indexed removal

`list remove index` shall remove exactly the entry at a valid index and preserve
the relative order of all other entries.

### TOPAL-LIST-REMOVE-INDEXES-001 — Predicate and range index removal

`list remove-indexes predicate` shall remove exactly indexes for which the
predicate returns true. The range form shall remove every included index and
shall reject a range extending outside the List.

### TOPAL-LIST-REMOVE-VALUES-001 — Predicate value removal

`list remove-values predicate` shall remove exactly entries for which the
predicate returns true while preserving the relative order of retained entries.

### TOPAL-LIST-ZIP-EXACT-001 — Equal-size zip

`left zip-exact right` shall pair entries at equal indexes. Unequal unchecked
runtime sizes shall return `out-of-range` from the lexical operation domain.

### TOPAL-LIST-ZIP-SHORTEST-001 — Truncating zip

`left zip-shortest right` shall pair entries through the shorter List's final
index and explicitly discard every unmatched suffix entry.

### TOPAL-LIST-ZIP-LONGEST-001 — Default-extending zip

`(left, left-default) zip-longest (right, right-default)` shall pair through the
longer List's final index and use only the corresponding conforming default for
each missing entry.

### TOPAL-LIST-UNZIP-001 — Pair sequence decomposition

`unzip pairs` shall return two Lists containing respectively the first and
second field at every index, preserving the input entry count in both.

### TOPAL-COLLECTION-FOREACH-001 — Finite List traversal

List `foreach` shall call its Unit action once per entry in List order, return
Unit, and leave the immutable source List available afterward.

### TOPAL-COLLECTION-ENTRIES-001 — Indexed List entry view

`list entries` shall return one `IndexedEntry T` record per List entry in order,
with zero-based `index` and unchanged `value` fields.

### TOPAL-COLLECTION-COLLECT-LIST-001 — List materialization

Unary `collect` over a finite List traversal shall materialize the same ordered
entries as a List without changing their classifier or multiplicity.

### TOPAL-COLLECTION-COLLECT-STRING-001 — Text materialization

`fragments collect String` shall concatenate Character or String Unicode
content in traversal order and return the resulting String.

### TOPAL-ARRAY-COLLECT-001 — Fixed-count Array collection

`source collect Array` over a finite List shall construct `Array N T`, where
`N` is the exact source entry count, preserving order and multiplicity.

### TOPAL-SET-COLLECT-001 — Unique unordered collection

`collect-set source` shall retain one representative of each equality class of
entries. Its observable Set value shall expose no ordering guarantee.

### TOPAL-BAG-COLLECT-001 — Multiplicity collection

`collect-bag source` shall retain each distinct value with its positive total
occurrence count and shall expose no ordering guarantee.

### TOPAL-MAP-COLLECT-001 — Explicit-collision Map collection

`collect-map pairs resolving policy` shall construct a Map from two-field key
and value products. `reject` diagnoses a duplicate key, `keep-first` retains its
first value, and `keep-last` retains its final value. Key equality and uniform
key/value classifiers are required.

### TOPAL-COLLECTION-ENTRY-COUNT-001 — Fundamental collection count

Array, Set, and Map entry count is their number of stored entries. Bag entry
count is the sum of occurrence counts rather than its distinct-value count.

### TOPAL-COLLECTION-EMPTY-PREDICATE-001 — Fundamental collection emptiness

Array, Set, Bag, and Map `empty?` shall be true exactly when their entry count
is zero.

### TOPAL-ARRAY-GET-CHECKED-001 — Checked unrefined Array access

`array-at? (array, index)` for `array : Array N T` and `index : Nat` SHALL
return `Some value` exactly when `index < N`, selecting the zero-based entry,
and SHALL otherwise return `None T`. It SHALL preserve `T` and SHALL NOT clamp,
wrap, or diagnose a nonnegative out-of-bounds index.

### TOPAL-MAP-LOOKUP-001 — Exact-key Map lookup

`map-lookup (mapping, key)` for `mapping : Map (K, V)` and `key : K` SHALL
return the unique associated value as `Optional V`, or `None V` when the key is
absent. A key outside exact classifier `K` SHALL be rejected before lookup.

### TOPAL-SET-CONTAINS-001 — Set membership

`set-contains? (members, value)` for `members : Set T` and `value : T` SHALL
report whether the value's equality class occurs in the Set. It SHALL expose no
storage or iteration order.

### TOPAL-BAG-MULTIPLICITY-001 — Bag occurrence count

`bag-multiplicity (bag, value)` for `bag : Bag T` and `value : T` SHALL return
its positive occurrence count, or zero when absent. It SHALL expose no storage
or iteration order.

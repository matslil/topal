use language (
  version is v0.1
)
use library std (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
starts-with? is std pattern starts-with?
ends-with? is std pattern ends-with?
contains? is std pattern contains?
replace-all is std pattern replace-all
subsequence? is std pattern subsequence?

values : List Int is Entry (1, Entry (2, Entry (3, Entry (4, Empty))))
consecutive : List Int is Entry (2, Entry (3, Empty))
gapped : List Int is Entry (1, Entry (3, Entry (4, Empty)))
reversed : List Int is Entry (3, Entry (1, Empty))

empty-prefix : Pass is Pass (starts-with? ("Topal", ""))
exact-prefix : Pass is Pass (starts-with? ("Topal", "Top"))
wrong-prefix : Pass is Pass (not (starts-with? ("Topal", "opal")))
exact-suffix : Pass is Pass (ends-with? ("Topal", "pal"))
text-contained : Pass is Pass (contains? ("Topal language", "language"))
text-absent : Pass is Pass (not (contains? ("Topal", "Rust")))
replacement : Pass is Pass ((replace-all ("one two one", ("one", "1"))) = "1 two 1")
list-consecutive : Pass is Pass (contains? (values, consecutive))
list-not-consecutive : Pass is Pass (not (contains? (values, gapped)))
list-subsequence : Pass is Pass (subsequence? (values, gapped))
subsequence-order : Pass is Pass (not (subsequence? (values, reversed)))

(empty-prefix, exact-prefix, wrong-prefix, exact-suffix, text-contained,
 text-absent, replacement, list-consecutive, list-not-consecutive,
 list-subsequence, subsequence-order)

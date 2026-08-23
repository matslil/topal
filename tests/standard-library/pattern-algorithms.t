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
count is std pattern count
find-all is std pattern find-all
split is std pattern split
glob? is std pattern glob?
regex-contains? is std pattern regex-contains?
contains-any? is std pattern contains-any?
join is std text join

values : List Int is Entry (1, Entry (2, Entry (3, Entry (4, Empty))))
consecutive : List Int is Entry (2, Entry (3, Empty))
gapped : List Int is Entry (1, Entry (3, Entry (4, Empty)))
reversed : List Int is Entry (3, Entry (1, Empty))
overlap-indexes : List Nat is Entry (0, Entry (1, Entry (2, Empty)))
split-parts : List String is Entry ("one", Entry ("two", Entry ("three", Empty)))
alternatives : List String is Entry ("Rust", Entry ("language", Empty))
split-result : List String is split ("one--two--three", "--")
split-result-joined is join (split-result, "|")

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
overlapping-count : Pass is Pass ((count ("aaaa", "aa")) = 3)
overlapping-indexes : Pass is Pass ((find-all ("aaaa", "aa")) = overlap-indexes)
exact-split : Pass is Pass (split-result-joined = "one|two|three")
glob-star : Pass is Pass (glob? ("topal-language", "t*lang?age"))
glob-whole-text : Pass is Pass (not (glob? ("topal", "opa")))
alternative-match : Pass is Pass (contains-any? ("Topal language", alternatives))
regex-match : Pass is Pass (regex-contains? ("Topal language", "T.pal +lang(uage)?"))
regex-absence : Pass is Pass (not (regex-contains? ("Topal", "^Rust$")))

(empty-prefix, exact-prefix, wrong-prefix, exact-suffix, text-contained,
 text-absent, replacement, list-consecutive, list-not-consecutive,
 list-subsequence, subsequence-order, overlapping-count, overlapping-indexes,
 exact-split, glob-star, glob-whole-text, alternative-match, regex-match,
 regex-absence)

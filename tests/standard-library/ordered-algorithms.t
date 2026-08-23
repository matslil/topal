use language (
  version is v0.1
)
use library std (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
sort is std ordered sort
sort-descending is std ordered sort-descending
lower-bound is std ordered lower-bound
upper-bound is std ordered upper-bound
equal-range is std ordered equal-range
binary-search is std ordered binary-search
merge is std ordered merge
smallest is std ordered smallest
nth is std ordered nth

values : List Int is Entry (4, Entry (1, Entry (3, Entry (1, Entry (2, Empty)))))
ascending : List Int is Entry (1, Entry (1, Entry (2, Entry (3, Entry (4, Empty)))))
descending : List Int is Entry (4, Entry (3, Entry (2, Entry (1, Entry (1, Empty)))))
none : List Int is Empty
rational-values : List Rational is Entry (Rational (3, 2), Entry (Rational (1, 2), Entry (Rational (1, 1), Empty)))
rational-ascending : List Rational is Entry (Rational (1, 2), Entry (Rational (1, 1), Entry (Rational (3, 2), Empty)))
merge-left : List Int is Entry (1, Entry (3, Entry (5, Empty)))
merge-right : List Int is Entry (2, Entry (4, Entry (6, Empty)))
merged : List Int is Entry (1, Entry (2, Entry (3, Entry (4, Entry (5, Entry (6, Empty))))))
smallest-three : List Int is Entry (1, Entry (1, Entry (2, Empty)))

index-is is fn (candidate : Optional Nat, expected : Int) -> Boolean
  candidate
    Some index then index = expected
    None then false
index-absent? is fn (candidate : Optional Nat) -> Boolean
  candidate
    Some index then false
    None then true
value-is is fn (candidate : Optional Int, expected : Int) -> Boolean
  candidate
    Some value then value = expected
    None then false
value-absent? is fn (candidate : Optional Int) -> Boolean
  candidate
    Some value then false
    None then true

ascending-order : Pass is Pass ((sort values) = ascending)
descending-order : Pass is Pass ((sort-descending values) = descending)
empty-sort : Pass is Pass ((sort none) = none)
lower-before-equals : Pass is Pass ((lower-bound (ascending, 1)) = 0)
upper-after-equals : Pass is Pass ((upper-bound (ascending, 1)) = 2)
middle-absence : Pass is Pass (empty? (equal-range (ascending, 5)))
ones is equal-range (ascending, 1)
ones-lower is range-lower ones
ones-upper is range-upper ones
equal-subrange : Pass is Pass ((ones-lower = 0) and (ones-upper = 2))
rational-sort : Pass is Pass ((sort rational-values) = rational-ascending)
binary-found : Pass is Pass (index-is (binary-search (ascending, 3), 3))
binary-absent : Pass is Pass (index-absent? (binary-search (ascending, 9)))
stable-merge : Pass is Pass ((merge (merge-left, merge-right)) = merged)
partial-selection : Pass is Pass ((smallest (values, 3)) = smallest-three)
nth-selection : Pass is Pass (value-is (nth (values, 2), 2))
nth-absence : Pass is Pass (value-absent? (nth (values, 20)))

(ascending-order, descending-order, empty-sort, lower-before-equals,
 upper-after-equals, middle-absence, equal-subrange, rational-sort,
 binary-found, binary-absent, stable-merge, partial-selection,
 nth-selection, nth-absence)

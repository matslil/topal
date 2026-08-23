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

values : List Int is Entry (4, Entry (1, Entry (3, Entry (1, Entry (2, Empty)))))
ascending : List Int is Entry (1, Entry (1, Entry (2, Entry (3, Entry (4, Empty)))))
descending : List Int is Entry (4, Entry (3, Entry (2, Entry (1, Entry (1, Empty)))))
none : List Int is Empty

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

(ascending-order, descending-order, empty-sort, lower-before-equals,
 upper-after-equals, middle-absence, equal-subrange)

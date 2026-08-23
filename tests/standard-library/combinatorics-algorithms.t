use language (
  version is v0.1
)
use library std (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
permutations is std combinatorics permutations
combinations is std combinatorics combinations
subsets is std combinatorics subsets
cartesian-product is std combinatorics cartesian-product

three : List Int is Entry (1, Entry (2, Entry (3, Empty)))
none : List Int is Empty
letters : List String is Entry ("a", Entry ("b", Empty))
numbers : List Int is Entry (1, Entry (2, Entry (3, Empty)))

six-permutations : Pass is Pass ((entry-count (permutations three)) = 6)
empty-permutation : Pass is Pass ((entry-count (permutations none)) = 1)
three-pairs : Pass is Pass ((entry-count (combinations (three, 2))) = 3)
zero-combination : Pass is Pass ((entry-count (combinations (three, 0))) = 1)
impossible-combination : Pass is Pass ((entry-count (combinations (three, 4))) = 0)
eight-subsets : Pass is Pass ((entry-count (subsets three)) = 8)
six-products : Pass is Pass ((entry-count (cartesian-product (letters, numbers))) = 6)
empty-product : Pass is Pass ((entry-count (cartesian-product (letters, none))) = 0)

(six-permutations, empty-permutation, three-pairs, zero-combination,
 impossible-combination, eight-subsets, six-products, empty-product)

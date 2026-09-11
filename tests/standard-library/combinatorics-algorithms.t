use language (
  version is v0.1
)
use library std (
  version is v0.1
)

Pass is Boolean constraint { value } value = true
factorial is std combinatorics factorial
subset-count is std combinatorics subset-count
permutations is std combinatorics permutations
combinations is std combinatorics combinations
subsets is std combinatorics subsets
cartesian-product is std combinatorics cartesian-product

three : List Int is Entry (1, Entry (2, Entry (3, Empty)))
none : List Int is Empty
letters : List String is Entry ("a", Entry ("b", Empty))
numbers : List Int is Entry (1, Entry (2, Entry (3, Empty)))
duplicate-values : List Int is Entry (1, Entry (1, Empty))
first-permutation : List Int is Entry (3, Entry (2, Entry (1, Empty)))
last-permutation : List Int is Entry (1, Entry (2, Entry (3, Empty)))
combination-one : List Int is Entry (1, Entry (2, Empty))
combination-two : List Int is Entry (1, Entry (3, Empty))
combination-three : List Int is Entry (2, Entry (3, Empty))
expected-combinations : List List Int is Entry (combination-one, Entry (combination-two, Entry (combination-three, Empty)))
expected-product : List (String, Int) is Entry (("a", 1), Entry (("a", 2), Entry (("a", 3), Entry (("b", 1), Entry (("b", 2), Entry (("b", 3), Empty))))))

zero-factorial : Pass is Pass ((factorial 0) = 1)
recursive-factorial : Pass is Pass ((factorial 5) = 120)
empty-subset-count : Pass is Pass ((subset-count none) = 1)
three-subset-count : Pass is Pass ((subset-count three) = 8)
six-permutations : Pass is Pass ((entry-count (permutations three)) = 6)
permutation-source-order : Pass is Pass (((first (permutations three)) = (Some first-permutation)) and ((first ((permutations three) reverse)) = (Some last-permutation)))
positional-duplicates-remain : Pass is Pass ((entry-count (permutations duplicate-values)) = 2)
empty-permutation : Pass is Pass ((entry-count (permutations none)) = 1)
three-pairs : Pass is Pass ((entry-count (combinations (three, 2))) = 3)
combination-source-order : Pass is Pass ((combinations (three, 2)) = expected-combinations)
zero-combination : Pass is Pass ((entry-count (combinations (three, 0))) = 1)
impossible-combination : Pass is Pass ((entry-count (combinations (three, 4))) = 0)
eight-subsets : Pass is Pass ((entry-count (subsets three)) = 8)
empty-subsets : Pass is Pass ((entry-count (subsets none)) = 1)
six-products : Pass is Pass ((entry-count (cartesian-product (letters, numbers))) = 6)
product-source-order : Pass is Pass ((cartesian-product (letters, numbers)) = expected-product)
empty-product : Pass is Pass ((entry-count (cartesian-product (letters, none))) = 0)
empty-left-product : Pass is Pass ((entry-count (cartesian-product ((Empty String), numbers))) = 0)

(zero-factorial, recursive-factorial, empty-subset-count, three-subset-count,
 six-permutations, permutation-source-order, positional-duplicates-remain,
 empty-permutation, three-pairs, combination-source-order, zero-combination,
 impossible-combination, eight-subsets, empty-subsets, six-products,
 product-source-order, empty-product, empty-left-product)

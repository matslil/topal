#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates construction of every fundamental homogeneous container from a
# finite List: Array retains order and count, Set removes duplicates, Bag counts
# them, and Map applies an explicit duplicate-key collision policy.
values : List Int is Entry ( 2, Entry ( 1, Entry ( 2, Empty ) ) )
pairs : List (String, Int) is Entry ( ("Ada", 10), Entry ( ("Lin", 8), Entry ( ("Ada", 11), Empty ) ) )
array is values collect Array
members is collect-set values
occurrences is collect-bag values
scores is collect-map pairs resolving keep-last
(array, members, occurrences, scores, entry-count array, entry-count members, entry-count occurrences, entry-count scores, empty? members, array-at? (array, 1), array-at? (array, 9), set-contains? (members, 2), bag-multiplicity (occurrences, 2), bag-multiplicity (occurrences, 9), map-lookup (scores, "Ada"), map-lookup (scores, "Grace"))

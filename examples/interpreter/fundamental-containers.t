#!/usr/bin/env topal
# Demonstrates construction of every fundamental homogeneous container from a
# finite List: Array retains order and count, Set removes duplicates, Bag counts
# them, and Map applies an explicit duplicate-key collision policy.
values : List Int is Entry ( 2, Entry ( 1, Entry ( 2, Empty ) ) )
pairs : List (String, Int) is Entry ( ("Ada", 10), Entry ( ("Lin", 8), Entry ( ("Ada", 11), Empty ) ) )
array is values collect Array
members is collect-set values
occurrences is collect-bag values
scores is collect-map pairs resolving keep-last
(array, members, occurrences, scores, entry-count array, entry-count members, entry-count occurrences, entry-count scores, empty? members)

#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates represented Array, Set, Bag, and Map values as complete package
# fields with reordered labeled calls and positional parity.
make-array is fn () -> Array (3, Int)
  values : List Int is Entry (2, Entry (1, Entry (2, Empty)))
  values collect Array

make-set is fn () -> Set Int
  values : List Int is Entry (2, Entry (1, Entry (2, Empty)))
  collect-set values

make-bag is fn () -> Bag Int
  values : List Int is Entry (2, Entry (1, Entry (2, Empty)))
  collect-bag values

make-map is fn () -> Map (String, Int)
  pairs : List (String, Int) is Entry (("Ada", 10), Entry (("Lin", 8), Entry (("Ada", 11), Empty)))
  collect-map pairs resolving keep-last

retain is fn ((array : Array (3, Int), members : Set Int, occurrences : Bag Int, scores : Map (String, Int))) -> (Array (3, Int), Set Int, Bag Int, Map (String, Int))
  (array, members, occurrences, scores)

(
  retain (scores is make-map (), occurrences is make-bag (), members is make-set (), array is make-array ()),
  retain (make-array (), make-set (), make-bag (), make-map ())
)

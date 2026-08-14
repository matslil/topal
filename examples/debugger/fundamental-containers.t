#!/usr/bin/env topal-debug
use language (
  version is v0.1
)
# Demonstrates reversible construction of Array, Set, Bag, and Map values from
# one immutable ordered List and an explicit map collision policy.
values : List Int is Entry ( 2, Entry ( 1, Entry ( 2, Empty ) ) )
pairs : List (String, Int) is Entry ( ("Ada", 10), Entry ( ("Ada", 11), Empty ) )
(values collect Array, collect-set values, collect-bag values, collect-map pairs resolving keep-last)

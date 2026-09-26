#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates returns from both actions of an exhaustive List decision.

choose is fn (candidate : List Int) -> Int
  candidate
    Entry (first, rest) then { return first + (entry-count rest) }
    Empty then { return 40 }
  1000

values : List Int is Entry (42, Entry (9, Empty))
empty-values : List Int is Empty
(choose values, choose empty-values)

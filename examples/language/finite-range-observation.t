#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates finite exact Range bound observation, empty predicates, and
# inclusivity preservation when equal endpoints are intersected.
integer is -2 <..= 3
rational is 0.5 .. 2
intersection is (0 ..= 2) and (0 <..= 3)
upper-intersection is (0 ..= 2) and (-1 .. 2)
choose is fn (condition : Boolean) -> Range Int
  condition
    true then 0 ..= 1
    otherwise 2 <.. 3
(empty? (3 .. 3), empty? (3 ..= 3), range-lower integer, range-upper integer, range-lower-inclusive? integer, range-upper-inclusive? integer, range-lower rational, range-upper rational, range-lower-inclusive? rational, range-upper-inclusive? rational, intersection, upper-intersection, choose true, empty? intersection)

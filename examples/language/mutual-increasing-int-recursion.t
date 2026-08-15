#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a mutual cycle whose every edge increases the same Int measure
# by one until a guarded upper bound is reached.
even-up is fn (value : Int) -> Boolean
  value
    >= 0 then true
    otherwise odd-up (value + 1)
odd-up is fn (value : Int) -> Boolean
  value
    >= 0 then false
    otherwise even-up (value + 1)
(even-up (-6), odd-up (-6))

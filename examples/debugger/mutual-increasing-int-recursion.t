#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible proof of a mutually increasing recursive cycle.
even-up is fn (value : Int) -> Boolean
  value
    >= 0 then true
    otherwise odd-up (value + 1)
odd-up is fn (value : Int) -> Boolean
  value
    >= 0 then false
    otherwise even-up (value + 1)
(even-up (-6), odd-up (-6))

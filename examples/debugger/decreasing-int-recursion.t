#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible proof, descent, nested entry, and recursive return.
sum-down is fn (value : Int) -> Int
  value
    <= 0 then 0
    otherwise value + (sum-down (value - 1))
sum-down 5

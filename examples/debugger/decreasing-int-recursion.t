#!/usr/bin/env topal
# Demonstrates reversible proof, descent, nested entry, and recursive return.
sum-down is fn (value : Int) -> Int
  value
    <= 0 then 0
    otherwise value + (sum-down (value - 1))
sum-down 5

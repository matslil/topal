#!/usr/bin/env topal
# Demonstrates reversible proof and recursive ascent toward an upper bound.
distance-up is fn (value : Int) -> Int
  value
    >= 0 then 0
    otherwise 1 + (distance-up (value + 1))
distance-up (-5)

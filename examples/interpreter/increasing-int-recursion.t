#!/usr/bin/env topal
# Demonstrates structurally proven recursion toward an upper bound: values at
# or above zero stop, while every lower recursive call passes value + 1.
distance-up is fn (value : Int) -> Int
  value
    >= 0 then 0
    otherwise 1 + (distance-up (value + 1))
distance-up (-5)

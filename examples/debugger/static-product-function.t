#!/usr/bin/env topal
# Demonstrates reversible binding and execution of a two-field function input.
add is fn static (left : Int, right : Int) -> Int
  left + right
add (20, 22)

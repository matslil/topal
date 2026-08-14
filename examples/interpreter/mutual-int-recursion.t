#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates two functions whose complete call cycle decreases the same Int
# measure by one until either guarded base case is selected.
even is fn (value : Int) -> Boolean
  value
    <= 0 then true
    otherwise odd (value - 1)
odd is fn (value : Int) -> Boolean
  value
    <= 0 then false
    otherwise even (value - 1)
(even 6, odd 6)

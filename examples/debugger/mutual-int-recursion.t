#!/usr/bin/env topal
# Demonstrates reversible proof and descent through a mutual recursive cycle.
even is fn (value : Int) -> Boolean
  value
    <= 0 then true
    otherwise odd (value - 1)
odd is fn (value : Int) -> Boolean
  value
    <= 0 then false
    otherwise even (value - 1)
(even 6, odd 6)

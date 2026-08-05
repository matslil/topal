#!/usr/bin/env topal
# Demonstrates a closed mutual Nat cycle whose every edge decrements by one.
even is fn (value : Nat) -> Boolean
  value
    <= 0 then true
    otherwise odd (value - 1)
odd is fn (value : Nat) -> Boolean
  value
    <= 0 then false
    otherwise even (value - 1)
(even 6, odd 6)

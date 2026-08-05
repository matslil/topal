#!/usr/bin/env topal
# Demonstrates a closed mutual Nat cycle whose bounded decrements preserve Nat.
even is fn (value : Nat) -> Boolean
  value
    <= 2 then true
    otherwise odd (value - 3)
odd is fn (value : Nat) -> Boolean
  value
    <= 2 then false
    otherwise even (value - 3)
(even 8, odd 8)

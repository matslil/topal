#!/usr/bin/env topal
# Demonstrates reversible mutual Nat recursion and closed-cycle proof.
even is fn (value : Nat) -> Boolean
  value
    <= 0 then true
    otherwise odd (value - 1)
odd is fn (value : Nat) -> Boolean
  value
    <= 0 then false
    otherwise even (value - 1)
(even 6, odd 6)

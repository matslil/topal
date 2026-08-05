#!/usr/bin/env topal
# Demonstrates reversible mutual increasing Nat recursion and cycle proof.
even is fn (value : Nat) -> Boolean
  value
    >= 6 then true
    otherwise odd (value + 1)
odd is fn (value : Nat) -> Boolean
  value
    >= 6 then false
    otherwise even (value + 1)
(even 0, odd 0)

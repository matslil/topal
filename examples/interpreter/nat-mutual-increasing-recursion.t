#!/usr/bin/env topal
# Demonstrates a closed mutual Nat cycle whose positive additions preserve Nat.
even is fn (value : Nat) -> Boolean
  value
    >= 6 then true
    otherwise odd (value + 1)
odd is fn (value : Nat) -> Boolean
  value
    >= 6 then false
    otherwise even (value + 1)
(even 0, odd 0)

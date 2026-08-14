#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible bounded-step mutual Nat recursion and cycle proof.
even is fn (value : Nat) -> Boolean
  value
    <= 2 then true
    otherwise odd (value - 3)
odd is fn (value : Nat) -> Boolean
  value
    <= 2 then false
    otherwise even (value - 3)
(even 8, odd 8)

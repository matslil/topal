#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates proven Nat recursion increasing by a positive literal step.
advance is fn (value : Nat) -> Nat
  value
    >= 5 then value
    otherwise advance (value + 2)
advance 0

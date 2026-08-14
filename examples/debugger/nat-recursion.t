#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible range-preserving Nat recursion and its proof.
count-down is fn (value : Nat) -> Nat
  value
    <= 2 then value
    otherwise count-down (value - 3)
count-down 8

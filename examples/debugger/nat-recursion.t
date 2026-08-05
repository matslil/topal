#!/usr/bin/env topal
# Demonstrates reversible unit-step Nat recursion and its termination proof.
count-down is fn (value : Nat) -> Nat
  value
    <= 0 then 0
    otherwise count-down (value - 1)
count-down 3

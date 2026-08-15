#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates proven Nat recursion whose bounded decrement cannot overshoot
# below the nonnegative inclusive base bound.
count-down is fn (value : Nat) -> Nat
  value
    <= 2 then value
    otherwise count-down (value - 3)
count-down 8

#!/usr/bin/env topal
# Demonstrates proven Nat recursion whose unit decrement cannot overshoot below
# the nonnegative inclusive base bound.
count-down is fn (value : Nat) -> Nat
  value
    <= 0 then 0
    otherwise count-down (value - 1)
count-down 3

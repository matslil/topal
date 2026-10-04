#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a recursive Euclidean GCD with a decreasing measure.
# Euclidean descent proves termination from the absolute value of the divisor.
# The zero branch also establishes that the recursive divisor is nonzero, so
# modulo remains total within the recursive branch.
gcd is fn (left : Int, right : Int) -> Nat : Decreases (absolute right)
  right
    = 0 then absolute left
    otherwise gcd (right, left % right)

gcd (84, -30)

#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that Nat keeps its exact Int value and comparison behavior after
# validation, including mixed exact comparison and derived product equality.
compare-nat is fn (left : Nat, right : Nat) -> Comparison
  left <=> right
zero : Nat is 0
one : Nat is 1
same-one : Nat is 1
large : Nat is 123456789012345678901234567890
same-large : Nat is 123456789012345678901234567890
tuple-left is (one, large)
tuple-same is (same-one, same-large)
tuple-different is (zero, large)
(zero = 0, one = 1.0, one != zero, one < large, large > one, one <= same-one, large >= same-large, compare-nat (one, large), tuple-left = tuple-same, tuple-left != tuple-different)

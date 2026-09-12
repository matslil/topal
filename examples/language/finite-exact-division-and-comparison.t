#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates finite Euclidean Int division, exact Rational division, and
# same-domain or canonically converted exact comparisons without machine bounds.
large is 123456789012345678901234567890
(17 % 5, -17 % 5, 17 % -5, -17 /% 5, 17 /% -5, large / 10, large % 97, 1 <=> 2, 2.5 <=> 2, 1 = 1.0, absolute -2.5, negate 2.5)

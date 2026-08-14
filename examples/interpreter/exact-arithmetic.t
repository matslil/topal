#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates arbitrary-precision integers, exact rationals, conversion, and powers.
large is 123456789012345678901234567890
fraction is 1.25
scaled is large + fraction
(scaled, 6 / 8, 2 ^ 16)

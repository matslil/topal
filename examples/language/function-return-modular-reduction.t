#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates return propagation after a named modular-reduction target check.
Counter is ModNat (0 ..= 255)

answer is fn (value : Int) -> Int
  abandoned : Counter is { return value + 1 } modulo Counter
  1000
answer 41

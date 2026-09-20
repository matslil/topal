#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates return propagation after named modular-type identity validation.
Counter is ModNat (0 ..= 255)

answer is fn (value : Int) -> Int
  abandoned : Counter is Counter { return value + 1 }
  1000
answer 41

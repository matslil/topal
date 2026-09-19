#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordered return propagation through direct symbolic operands.
preceding is fn (value : Int) -> Int
  value + 100
right-exit is fn (value : Int) -> Int
  preceding value + { return value + 1 } + missing-right
  1000
left-exit is fn (value : Int) -> Int
  { return value + 2 } + missing
  1000
(right-exit 41, left-exit 41)

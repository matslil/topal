#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordered return propagation through direct named-call arguments.
preceding is fn (value : Int) -> Int
  value + 100
combine is fn (left : Int, right : Int) -> Int
  left + right
identity is fn (value : Int) -> Int
  value
right-exit is fn (value : Int) -> Int
  preceding value combine { return value + 1 }
  1000
left-exit is fn (value : Int) -> Int
  { return value + 2 } combine missing-right
  1000
unary-exit is fn (value : Int) -> Int
  identity { return value + 3 }
  1000
(right-exit 41, left-exit 41, unary-exit 41)

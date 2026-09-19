#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordered return propagation through direct product fields.
preceding is fn (value : Int) -> Int
  value + 100
tuple-exit is fn (value : Int) -> Int
  (preceding value, preceding value, { return value + 1 }, missing-tuple)
  1000
record-exit is fn (value : Int) -> Int
  (first is preceding value, second is preceding value, exit is { return value + 2 }, missing is missing-record)
  1000
(tuple-exit 41, record-exit 41)

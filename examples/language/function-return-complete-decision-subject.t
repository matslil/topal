#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates returns before complete no-fallback decision matching.

optional-exit is fn (value : Int) -> Int
  { return value + 1 }
    Some payload then payload
    None then 0
  1000

result-exit is fn (value : Int) -> Int
  { return value + 1 }
    Ok payload then payload
    Error problem then 0
  1000

list-exit is fn (value : Int) -> Int
  { return value + 1 }
    Empty then 0
    Entry (first, rest) then first
  1000
(optional-exit 39, result-exit 40, list-exit 41)

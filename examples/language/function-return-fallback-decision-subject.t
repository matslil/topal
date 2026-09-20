#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a return before fallback-complete decision matching.

answer is fn (value : Int) -> Int
  { return value + 1 }
    Some payload then payload
    Error problem then 0
    Empty then 1
    otherwise 2
  1000
answer 41

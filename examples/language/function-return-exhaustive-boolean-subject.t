#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a return before exhaustive Boolean decision matching.

answer is fn (value : Int) -> Int
  { return value + 1 }
    false then false
    true then true
  1000
answer 41

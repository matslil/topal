#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a return before Boolean decision subject classification.

answer is fn (value : Int) -> Int
  { return value + 1 }
    true then false
    otherwise true
  1000
answer 41

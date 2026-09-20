#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a return before ordered comparison decision matching.

answer is fn (value : Int) -> Int
  { return value + 1 }
    < 0 then 0
    = 1 then 1
    otherwise 2
  1000
answer 41

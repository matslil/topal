#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible multiple-edge descent through a mutual cycle.
first-count is fn (value : Int) -> Int
  value
    <= 0 then 1
    otherwise (second-count (value - 1)) + (second-count (value - 2))
second-count is fn (value : Int) -> Int
  value
    <= 0 then 1
    otherwise first-count (value - 1)
first-count 3

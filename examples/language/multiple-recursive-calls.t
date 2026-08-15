#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that every recursive call inside one action is checked; both
# branches decrease by a positive literal amount before their results combine.
branch-count is fn (value : Int) -> Int
  value
    <= 0 then 1
    otherwise (branch-count (value - 1)) + (branch-count (value - 2))
branch-count 3

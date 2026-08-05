#!/usr/bin/env topal
# Demonstrates reversible descent through two proven calls in one action.
branch-count is fn (value : Int) -> Int
  value
    <= 0 then 1
    otherwise (branch-count (value - 1)) + (branch-count (value - 2))
branch-count 3

#!/usr/bin/env topal
# Demonstrates a mutual cycle where one action calls the next member twice;
# every call targets the same member and independently decreases its argument.
first-count is fn (value : Int) -> Int
  value
    <= 0 then 1
    otherwise (second-count (value - 1)) + (second-count (value - 2))
second-count is fn (value : Int) -> Int
  value
    <= 0 then 1
    otherwise first-count (value - 1)
first-count 3

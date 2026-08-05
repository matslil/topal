#!/usr/bin/env topal
# Demonstrates reversible local shadowing without changing the outer binding.
value is 40
answer is fn () -> Int
  value is 42
  value
(answer (), value)

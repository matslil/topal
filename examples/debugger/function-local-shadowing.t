#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible local shadowing without changing the outer binding.
value is 40
answer is fn () -> Int
  value is 42
  value
(answer (), value)

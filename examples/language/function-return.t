#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that explicit return completes the function with its expression
# and prevents the later body expression from being evaluated.
answer is fn static () -> Int
  return 40 + 2
  0
answer ()

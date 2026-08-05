#!/usr/bin/env topal
# Demonstrates that explicit return completes the function with its expression
# and prevents the later body expression from being evaluated.
answer is fn static () -> Int
  return 40 + 2
  0
answer ()

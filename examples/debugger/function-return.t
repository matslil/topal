#!/usr/bin/env topal
# Demonstrates reversible explicit return and an unreachable body expression.
answer is fn static () -> Int
  return 40 + 2
  0
answer ()

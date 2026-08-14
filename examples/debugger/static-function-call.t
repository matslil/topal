#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible decisions inside a static zero-parameter function call.
answer is fn static () -> Int
  40 + 2
answer ()

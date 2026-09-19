#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that a return inside an unconditional lexical block completes
# the surrounding function without evaluating the outer tail.
answer is fn (value : Int) -> Int
  adjusted is value + 1
  { return adjusted }
  1000
answer 41

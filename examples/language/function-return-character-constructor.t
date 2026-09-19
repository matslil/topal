#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates return propagation before Character constraint validation.
answer is fn (value : Int) -> Int
  abandoned : Character is Character { return value + 1 }
  1000
answer 41

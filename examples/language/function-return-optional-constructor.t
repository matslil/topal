#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that a Some payload return abandons classification and construction.
answer is fn (value : Int) -> Int
  abandoned : Optional String is Some { return value + 1 }
  1000
answer 41

#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates return propagation after Variant type and index validation.
Choice is Variant (String, Int)

answer is fn (value : Int) -> Int
  abandoned : Choice is Choice at 0 { return value + 1 }
  1000
answer 41

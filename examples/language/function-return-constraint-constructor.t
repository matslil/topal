#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates return propagation after named-constraint identity validation.
Positive is Int constraint { candidate } candidate > 0

answer is fn (value : Int) -> Int
  abandoned : Positive is Positive { return value + 1 }
  1000
answer 41

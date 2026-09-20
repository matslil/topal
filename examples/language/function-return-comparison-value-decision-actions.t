#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates returns from every action of a complete Comparison decision.

choose is fn (value : Comparison) -> Int
  value
    Less then { return 40 }
    Equal then { return 41 }
    Greater then { return 42 }
  1000
(choose (1 <=> 2), choose (2 <=> 2), choose (3 <=> 2))

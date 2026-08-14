#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible exact comparison and exhaustive Comparison decisions.
describe is fn (value : Comparison) -> String
  value
    Less then "less"
    Equal then "equal"
    Greater then "greater"
(1 <=> 2, 2 <=> 2, 3 <=> 2, 1 <=> 1.5, describe (1 <=> 2), describe (2 <=> 2), describe (3 <=> 2))

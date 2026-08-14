#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact three-way comparison for Int, Rational, and their canonical
# mixed-domain conversion, then exhaustively consumes Comparison alternatives.
describe is fn (value : Comparison) -> String
  value
    Less then "less"
    Equal then "equal"
    Greater then "greater"
(1 <=> 2, 2 <=> 2, 3 <=> 2, 1 <=> 1.5, describe (1 <=> 2), describe (2 <=> 2), describe (3 <=> 2))

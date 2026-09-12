#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exhaustive Comparison alternative matching and ordered matcher
# operands that are complete, lazily reached mixed exact expressions.
rank is fn (value : Comparison) -> Int
  value
    Less then -1
    Equal then 0
    Greater then 1
locate is fn (value : Int, pivot : Rational) -> Int
  value
    < pivot - 0.5 then -1
    = pivot then 0
    otherwise 1
below-one is fn (value : Rational) -> Boolean
  value
    < 1 then true
    otherwise false
(rank (1 <=> 2), rank (2 <=> 2), rank (3 <=> 2), 0 locate 1.5, 1 locate 1.0, 2 locate 1.5, below-one 0.5, below-one 1.5)

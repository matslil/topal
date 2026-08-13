#!/usr/bin/env topal
# Demonstrates symbolic operators as function values. Binary operands are
# packaged in a positional product; bound minus also accepts unary negation.
add is +
negate is -
compare-values is <=>
(add (20, 22), negate 5, compare-values (3, 4))

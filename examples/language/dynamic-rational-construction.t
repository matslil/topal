#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates dynamic Rational construction: a nonzero denominator succeeds,
# nonzero divided by directionless zero fails, and (0, 0) is indeterminate.
ratio is fn (numerator : Int, denominator : Int) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  Rational (numerator, denominator)
(1 ratio 2, 1 ratio 0, 0 ratio 0)

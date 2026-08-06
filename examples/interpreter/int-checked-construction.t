#!/usr/bin/env topal
# Demonstrates exact checked Int construction: an Int is preserved, an exact
# dynamic Rational becomes Int, and a fractional dynamic Rational returns Error.
as-int is fn (value : Rational) -> Result (Int, lang arithmetic ArithmeticErrorCode)
  Int value
(Int 7, as-int 6.0, as-int 1.5)

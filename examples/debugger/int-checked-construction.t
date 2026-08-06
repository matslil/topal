#!/usr/bin/env topal
# Demonstrates reversible exact checked Int construction and validation failure.
as-int is fn (value : Rational) -> Result (Int, lang arithmetic ArithmeticErrorCode)
  Int value
(Int 7, as-int 6.0, as-int 1.5)

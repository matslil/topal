#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible construction of a negative-power arithmetic Error.
power is fn (base : Rational, exponent : Int) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  base ^ exponent
0.0 power -1

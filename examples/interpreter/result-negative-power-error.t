#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a dynamic zero Rational base returning division-by-zero for a
# negative exponent, with domain derived from the reporting power overload.
power is fn (base : Rational, exponent : Int) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  base ^ exponent
0.0 power -1

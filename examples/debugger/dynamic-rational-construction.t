#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible dynamic Rational success and both zero failures.
ratio is fn (numerator : Int, denominator : Int) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  Rational (numerator, denominator)
(1 ratio 2, 1 ratio 0, 0 ratio 0)

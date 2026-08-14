#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible construction of a dynamic division Error.
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
1.0 divide 0.0

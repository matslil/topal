#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a structured arithmetic Error propagating through another
# fallible function without changing its code, domain, or source provenance.
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
outer is fn () -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  1.0 divide 0.0
outer ()

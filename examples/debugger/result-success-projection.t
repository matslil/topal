#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible success projection and early Error propagation.
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
increment-quotient is fn (denominator : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  quotient : Rational is 1.0 divide denominator
  quotient + 1.0
(increment-quotient 2.0, increment-quotient 0.0)

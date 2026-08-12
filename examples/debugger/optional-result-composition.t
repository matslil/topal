#!/usr/bin/env topal-debug
# Demonstrates reversible Optional decisions and Result propagation while
# preserving the complete structured Error and its generated source location.
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
propagate is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  value : Rational is left divide right
  value
attempt is 1.0 propagate 0.0
(attempt domain, attempt code, attempt detail, attempt cause, attempt source)

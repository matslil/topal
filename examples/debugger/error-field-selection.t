#!/usr/bin/env topal
# Demonstrates reversible selection of an Error code and reporting domain.
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
problem is 1.0 divide 0.0
(problem code, problem domain)

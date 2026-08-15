#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that an Error exposes its namespace-defined code and its
# compiler-derived reporting domain as separate, typed fields.
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
problem is 1.0 divide 0.0
(problem code, problem domain)

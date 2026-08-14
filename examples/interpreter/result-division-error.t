#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates dynamic Rational division by zero returning an Error whose domain
# derives from the reporting root overload; the code comes from lang arithmetic.
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
1.0 divide 0.0

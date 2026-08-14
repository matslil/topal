#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible selection of a qualified Error-code pattern.
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
describe is fn (denominator : Rational) -> String
  1.0 divide denominator
    Ok value then "ok"
    Error ( code is lang arithmetic division-by-zero ) then "division by zero"
    Error problem then "other arithmetic error"
(describe 2.0, describe 0.0)

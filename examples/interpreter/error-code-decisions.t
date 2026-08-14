#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a qualified Error-code pattern, followed by a whole-Error
# fallback; the code is matched independently of the reporting domain.
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
describe is fn (denominator : Rational) -> String
  1.0 divide denominator
    Ok value then "ok"
    Error ( code is lang arithmetic division-by-zero ) then "division by zero"
    Error problem then "other arithmetic error"
(describe 2.0, describe 0.0)

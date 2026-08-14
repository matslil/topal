#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exhaustive Result matching with action-scoped success and Error
# payload bindings; unselected actions remain unevaluated.
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
describe is fn (denominator : Rational) -> String
  1.0 divide denominator
    Ok value then "ok"
    Error problem then "error"
(describe 2.0, describe 0.0)

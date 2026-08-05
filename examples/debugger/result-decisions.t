#!/usr/bin/env topal
# Demonstrates reversible Result matcher selection and payload binding.
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
describe is fn (denominator : Rational) -> String
  1.0 divide denominator
    Ok value then "ok"
    Error problem then "error"
(describe 2.0, describe 0.0)

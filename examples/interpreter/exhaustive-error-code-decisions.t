#!/usr/bin/env topal
# Demonstrates that Ok plus every qualified member of the closed arithmetic
# error vocabulary is exhaustive without a generic Error fallback.
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
describe is fn (denominator : Rational) -> String
  1.0 divide denominator
    Ok value then "ok"
    Error ( code is lang arithmetic out-of-range ) then "range"
    Error ( code is lang arithmetic not-representable ) then "representation"
    Error ( code is lang arithmetic division-by-zero ) then "zero"
    Error ( code is lang arithmetic indeterminate ) then "indeterminate"
(describe 2.0, describe 0.0)

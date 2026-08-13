#!/usr/bin/env topal
# Demonstrates complete Optional and Result decisions, contextual success
# projection, propagation, and every structured Error field. This generated
# arithmetic error has absent detail/cause and a present source location.
describe-optional is fn (value : Optional Int) -> Int
  value
    Some present then present
    None then 0

divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right

propagate is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  value : Rational is left divide right
  value

attempt is 1.0 propagate 0.0
(describe-optional (Some 4), describe-optional (None Int), attempt domain, attempt code, attempt detail, attempt cause, attempt source)

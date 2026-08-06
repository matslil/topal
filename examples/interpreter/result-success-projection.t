#!/usr/bin/env topal
# Demonstrates contextual Result projection: the classified binding receives a
# success value, while an Error returns immediately from the enclosing function.
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
increment-quotient is fn (denominator : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  quotient : Rational is 1.0 divide denominator
  quotient + 1.0
(increment-quotient 2.0, increment-quotient 0.0)

#!/usr/bin/env topal
# Demonstrates checked dynamic Rational-to-Int validation: exact quotients
# become Int, while a fractional quotient propagates not-representable.
halve is fn (value : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)
  half : Int is value / 2
  half
(halve 100, halve 3)

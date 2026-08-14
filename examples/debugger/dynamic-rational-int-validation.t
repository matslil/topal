#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible checked Rational-to-Int success and failure.
halve is fn (value : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)
  half : Int is value / 2
  half
(halve 100, halve 3)

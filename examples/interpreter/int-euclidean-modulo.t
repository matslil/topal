#!/usr/bin/env topal
# Demonstrates Euclidean Int modulo: every nonzero divisor produces a
# nonnegative remainder below its absolute value; dynamic zero returns Error.
modulo is fn (left : Int, right : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)
  left % right
(17 % 5, -17 % 5, 17 % -5, 17 modulo 0)

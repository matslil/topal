#!/usr/bin/env topal
# Demonstrates reversible Euclidean modulo and dynamic zero failure.
modulo is fn (left : Int, right : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)
  left % right
(17 % 5, -17 % 5, 17 % -5, 17 modulo 0)

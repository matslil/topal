#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible Euclidean modulo and dynamic zero failure.
modulo is fn (left : Int, right : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)
  left % right
quotient-modulo is fn (left : Int, right : Int) -> Result ((Int, Int), lang arithmetic ArithmeticErrorCode)
  left /% right
(17 % 5, -17 % 5, 17 % -5, -17 /% 5, 17 /% -5, 17 modulo 0, 17 quotient-modulo 0)

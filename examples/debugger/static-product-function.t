#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible infix-operand and local binding inside a function block.
add is fn static (left : Int, right : Int) -> Int
  sum is left + right
  sum
20 add 22

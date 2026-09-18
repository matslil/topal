#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates label-based association for a packaged function operand. Supplied
# expressions execute in source order before fields bind in declaration order.
right-value is fn () -> Int
  2

left-value is fn () -> Int
  39

combine is fn ((left : Int, offset : Int default 1, right : Int)) -> Int
  left + offset + right

(
  combine (right is right-value (), left is left-value ()),
  combine (right is 2, offset is 3, left is 37),
  combine (39, 1, 2)
)

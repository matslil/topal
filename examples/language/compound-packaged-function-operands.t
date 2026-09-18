#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates two packaged operands and packages mixed with ordinary operands.
# Explicit values execute once in source order before defaults bind in operand
# and field declaration order.
left-value is fn () -> Int
  20

right-value is fn () -> Int
  21

scale-value is fn () -> Int
  20

scale-factor is fn () -> Int
  2

combine is fn ((left : Int, left-offset : Int default 1), (right : Int, right-offset : Int default 0)) -> Int
  left + left-offset + right + right-offset

scale is fn ((value : Int, offset : Int default 2), factor : Int) -> Int
  value * factor + offset

shift is fn (base : Int, (amount : Int, offset : Int default 1)) -> Int
  base + amount + offset

(
  (left-offset is 1, left is left-value ()) combine (right is right-value ()),
  (left is 20) combine (right-offset is 1, right is 20),
  (20, 1) combine (21, 0),
  (value is scale-value ()) scale (scale-factor ()),
  20 shift (amount is 21)
)

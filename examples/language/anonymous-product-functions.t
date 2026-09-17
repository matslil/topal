#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact private destructuring for direct, capturing, returned, and
# mixed-parameter anonymous functions without a closure or aggregate runtime.
make-pair is fn (left : Int, right : Int) -> (Int, Int)
  (left, right)

apply-offset is fn (offset : Int, values : (Int, Int)) -> Int
  operation : Function is { (left, right) } left + right + offset
  operation values

make-sum is fn () -> Function
  { (left, right) } left + right

direct : Function is { (left, right) } left + right
returned is make-sum ()
mixed : Function is { (left, right), extra } left + right + extra
(direct (make-pair (20, 22)), apply-offset (1, (20, 21)), returned (19, 23), mixed ((10, 20), 12))

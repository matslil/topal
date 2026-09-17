#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates recursive product destructuring in inferred anonymous Functions.
make-values is fn (base : Int) -> (Int, (Int, Int))
  (base, (base + 1, base + 2))

nested : Function is { (left, (middle, right)) } left + middle + right

captured is fn (offset : Int) -> Int
  operation : Function is { ((left, right), tail) } left + right + tail + offset
  operation ((10, 11), 20)

make-nested is fn (offset : Int) -> Function
  { (left, (middle, right)) } left + middle + right + offset

mixed : Function is { (left, (middle, right)), extra } left + middle + right + extra

repeated : Function is { (value, (value, _)) } value

(
  nested (make-values 13),
  captured 1,
  (make-nested 1) (10, (11, 20)),
  mixed ((10, (11, 20)), 1),
  repeated (42, (42, 0))
)

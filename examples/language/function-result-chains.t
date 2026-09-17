#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates left-associative application of exact Function results without
# binding the intermediate callable value.
increment is fn (value : Int) -> Int
  value + 1

return-operation is fn (operation : Function) -> Function
  operation

make-offset is fn (offset : Int) -> Function
  { value } value + offset

make-pair is fn (pair : (Int, String)) -> Function
  { ignored } pair

make-closed is fn () -> Function
  { value } value + 1

(
  make-offset 1 41,
  (make-offset 2) 40,
  make-pair (7, "seven") 0,
  make-closed () 41,
  (return-operation increment) 41,
  (return-operation +) (20, 22)
)

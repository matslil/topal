#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates an exact Function value as one complete package field.
make-operation is fn () -> Function
  +

make-value is fn () -> Int
  40

apply is fn ((operation : Function default +, value : Int default 40)) -> Int
  operation (value, 2)

offset is 2
add-offset : Function is { value } value + offset

apply-captured is fn ((operation : Function), value : Int) -> Int
  operation value

(
  apply (value is make-value (), operation is make-operation ()),
  apply (value is 40),
  apply (+, 40),
  (operation is add-offset) apply-captured 40
)

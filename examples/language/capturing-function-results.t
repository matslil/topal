#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates private captured Function results as exact tag-plus-capture
# aggregates with direct application after binding and no closure runtime.
return-operation is fn (operation : Function) -> Function
  operation

make-scalars is fn (left : Int, right : Int) -> Function
  { value } value + left + right

make-pair is fn (pair : (Int, String)) -> Function
  { ignored } pair

make-forwarded is fn (left : Int, right : Int) -> Function
  operation : Function is { value } value + left + right
  return-operation operation

root-offset is 40
scalar-operation is make-scalars (1, 2)
pair-operation is make-pair (7, "seven")
forwarded-operation is make-forwarded (1, 2)
root-operation is make-scalars (root-offset, 2)

(scalar-operation 39, pair-operation 0, forwarded-operation 39, root-operation 0)

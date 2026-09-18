#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates private Function-parameter closure conversion with exact hidden
# capture forwarding and no public closure representation or indirect dispatch.
apply-int is fn (operation : Function, value : Int) -> Int
  operation value

forward-int is fn (operation : Function, value : Int) -> Int
  apply-int (operation, value)

apply-pair is fn (operation : Function, value : Int) -> (Int, String)
  operation value

capture-scalars is fn (left : Int, right : Int) -> Int
  operation : Function is { input } input + left + right
  forward-int (operation, 39)

capture-pair is fn (pair : (Int, String), value : Int) -> (Int, String)
  operation : Function is { ignored } pair
  apply-pair (operation, value)

capture-nested is fn (offset : Int, value : Int) -> Int
  add is fn (input : Int) -> Int
    input + offset
  forward-int (add, value)

root-offset is 40
root-operation : Function is { value } value + root-offset

(capture-scalars (1, 2), forward-int (root-operation, 2), capture-pair ((7, "seven"), 0), capture-nested (2, 40))

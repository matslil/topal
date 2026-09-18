#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact Function values retained inside private Tuple and Record
# bindings, parameters, results, and nested aggregate fields.
increment is fn (value : Int) -> Int
  value + 1

make-tuple is fn (operation : Function, value : Int) -> (Function, Int)
  (operation, value)

apply-tuple is fn (package : (Function, Int)) -> Int
  invoke : Function is { (operation, value) } operation value
  invoke package

make-record is fn (operation : Function, value : Int) -> Record (operation : Function, value : Int)
  (operation is operation, value is value)

apply-record is fn (package : Record (operation : Function, value : Int)) -> Int
  (package operation) (package value)

make-nested is fn (operation : Function, value : Int) -> Record (payload : (Function, Int))
  (payload is (operation, value))

apply-nested is fn (package : Record (payload : (Function, Int))) -> Int
  invoke : Function is { (operation, value) } operation value
  invoke (package payload)

offset is 2
captured : Function is { value } value + offset
local is (operation is captured, value is 40)

(
  (local operation) (local value),
  apply-tuple (make-tuple (increment, 41)),
  apply-record (make-record (-, -42)),
  apply-nested (make-nested ({ value } value + value, 21))
)

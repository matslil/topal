#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact nested Function values escaping private factory frames
# with immutable lexical, defining-context, and live-root capture snapshots.
context-offset is 40

return-operation is fn (operation : Function) -> Function
  operation

forward-record is fn (package : Record (operation : Function, value : Int)) -> Record (operation : Function, value : Int)
  package

apply-record is fn (package : Record (operation : Function, value : Int)) -> Int
  (package operation) (package value)

make-operation is fn (offset : Int) -> Function
  increase is fn (value : Int) -> Int
    value + offset + @ context-offset + (root live-offset)
  increase

make-record is fn (offset : Int, value : Int) -> Record (operation : Function, value : Int)
  increase is fn (operand : Int) -> Int
    operand + offset + @ context-offset + (root live-offset)
  (operation is increase, value is value)

make-pair-operation is fn (pair : (Int, String)) -> Function
  read-pair is fn (ignored : Int) -> (Int, String)
    pair
  read-pair

live-offset is 1
first-operation is make-operation 1
second-operation is make-operation 2
forwarded-operation is return-operation (make-operation 3)
pair-operation is make-pair-operation (7, "seven")
package is forward-record (make-record (4, 1))

(
  first-operation 1,
  second-operation 1,
  forwarded-operation 1,
  apply-record package,
  make-operation 5 1,
  pair-operation 0
)

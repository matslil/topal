#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact captured Function values transported through private
# Tuple and Record parameters/results without a closure runtime.
make-record is fn (offset : Int, factor : Int) -> Record (operation : Function, scale : Function, value : Int)
  operation : Function is { value } value + offset
  scale : Function is { value } value * factor
  (operation is operation, scale is scale, value is 20)

apply-record is fn (package : Record (operation : Function, scale : Function, value : Int)) -> (Int, Int)
  ((package operation) (package value), (package scale) (package value))

forward-record is fn (package : Record (operation : Function, scale : Function, value : Int)) -> Record (operation : Function, scale : Function, value : Int)
  package

make-tuple is fn (offset : Int) -> (Function, Int)
  operation : Function is { value } value + offset
  (operation, 40)

apply-tuple is fn (package : (Function, Int)) -> Int
  invoke : Function is { (operation, value) } operation value
  invoke package

apply-one is fn (package : Record (operation : Function, value : Int)) -> Int
  (package operation) (package value)

use-nested is fn (offset : Int) -> Int
  increase is fn (value : Int) -> Int
    value + offset
  package is (operation is increase, value is 40)
  apply-one package

(
  apply-record (make-record (22, 2)),
  apply-record (forward-record (make-record (22, 2))),
  apply-tuple (make-tuple 2),
  use-nested 2
)

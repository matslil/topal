#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact Function-containing Tuple and Record values as complete
# package fields while preserving source order and private capture transport.
make-addend is fn () -> Int
  2

make-record is fn () -> Record (operation : Function, value : Int)
  (operation is +, value is 40)

apply-record is fn ((bundle : Record (operation : Function, value : Int), addend : Int)) -> Int
  (bundle operation) ((bundle value), addend)

make-tuple is fn (offset : Int) -> (Function, Int)
  operation : Function is { value } value + offset
  (operation, 40)

apply-tuple is fn ((bundle : (Function, Int), marker : Int default 0)) -> Int
  invoke : Function is { (operation, value) } operation value
  (invoke bundle) + marker

apply-default is fn ((bundle : Record (operation : Function, value : Int) default (operation is +, value is 40), addend : Int)) -> Int
  (bundle operation) ((bundle value), addend)

(
  apply-record (addend is make-addend (), bundle is make-record ()),
  apply-tuple (marker is 0, bundle is make-tuple 2),
  apply-default (addend is 2)
)

#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact repeated-name identity for capture-free Function leaves
# inside private Tuple and Record values.
increment is fn (value : Int) -> Int
  value + 1

make-tuple is fn (operation : Function, value : Int) -> (Function, Int)
  (operation, value)

make-record is fn (operation : Function, value : Int) -> Record (operation : Function, value : Int)
  (operation is operation, value is value)

repeat-tuple : Function is { value, value } 42
repeat-record : Function is { value, value } 42
repeat-nested : Function is { value, value } 42

tuple-value is make-tuple (increment, 41)
record-value is make-record (-, -42)
nested-value is (payload is ({ value } value + 1, 41))

(
  repeat-tuple (tuple-value, tuple-value),
  repeat-record (record-value, record-value),
  repeat-nested (nested-value, nested-value)
)

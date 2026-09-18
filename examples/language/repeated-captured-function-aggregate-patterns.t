#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that repeated Tuple and Record values compare captured Function
# leaves by exact callable identity and represented capture payloads.
make-record is fn (offset : Int) -> Record (operation : Function, value : Int)
  operation : Function is { value } value + offset
  (operation is operation, value is 41)

make-nested is fn (offset : Int) -> Record (payload : (Function, Int))
  operation : Function is { value } value + offset
  (payload is (operation, 41))

compare-records is fn (left : Int, right : Int) -> Int
  repeat : Function is { package, package } 42
  repeat (make-record left, make-record right)

compare-nested is fn (left : Int, right : Int) -> Int
  repeat : Function is { package, package } 42
  repeat (make-nested left, make-nested right)

compare-named is fn (offset : Int) -> Int
  increase is fn (value : Int) -> Int
    value + offset
  package is (operation is increase, value is 41)
  repeat : Function is { value, value } 42
  repeat (package, package)

(compare-records (1, 1), compare-nested (21, 21), compare-named 1)

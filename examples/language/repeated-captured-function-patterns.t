#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that repeated captured Function values compare both their exact
# callable identity and their already-evaluated represented capture payloads.
make-operation is fn (offset : Int) -> Function
  operation : Function is { value } value + offset
  operation

compare-captures is fn (left : Int, right : Int) -> Int
  repeat : Function is { operation, operation } 42
  repeat (make-operation left, make-operation right)

(compare-captures (1, 1), compare-captures (21, 21))

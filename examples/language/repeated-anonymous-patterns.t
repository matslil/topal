#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact repeated-name matching without conversion, re-evaluation,
# duplicate source bindings, or a pattern runtime.
make-pair is fn (left : Int, right : Int) -> (Int, Int)
  (left, right)

repeat-product : Function is { (value, value) } value
repeat-parameters : Function is { value, value } value
repeat-text : Function is { (text, text) } text
repeat-function : Function is { (operation, operation) } operation (6, 7)

(repeat-product (42, 42), repeat-product (make-pair (20, 20)), repeat-parameters (7, 7), repeat-text ("same", "same"), repeat-function (*, *))

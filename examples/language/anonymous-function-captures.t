#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a non-escaping anonymous function capturing immutable data and a
# non-capturing anonymous function returned through a private Function result.
apply-offset is fn (offset : Int, value : Int) -> Int
  operation : Function is { input } input + offset
  operation value

make-double is fn () -> Function
  { value } value + value

twice is make-double ()
(apply-offset (1, 41), twice 21)

#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible capture and invocation of a named function value after
# rebinding it under a different local name.
increment is fn (value : Int) -> Int
  value + 1

operation is increment
operation 41

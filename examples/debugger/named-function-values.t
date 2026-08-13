#!/usr/bin/env topal
# Demonstrates reversible capture and invocation of a named function value after
# rebinding it under a different local name.
increment is fn (value : Int) -> Int
  value + 1

operation is increment
operation 41

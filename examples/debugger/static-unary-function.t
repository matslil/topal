#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible argument binding and execution inside a static call.
increment is fn static (input : Int) -> Int
  input + 1
increment 41

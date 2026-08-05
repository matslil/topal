#!/usr/bin/env topal
# Demonstrates reversible argument binding and execution inside a static call.
increment is fn static (input : Int) -> Int
  input + 1
increment 41

#!/usr/bin/env topal
# Demonstrates reversible nested entry and return to a later declaration.
answer is fn () -> Int
  increment 41
increment is fn (input : Int) -> Int
  input + 1
answer ()

#!/usr/bin/env topal
# Demonstrates an ordinary function calling a later-declared ordinary function.
# Each nested call receives its own typed parameter binding and result validation.
answer is fn () -> Int
  increment 41
increment is fn (input : Int) -> Int
  input + 1
answer ()

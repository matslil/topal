#!/usr/bin/env topal
# Demonstrates an invocation-local helper capturing the outer input. The helper
# is declared and called inside `answer`; its name does not escape that call.
answer is fn (input : Int) -> Int
  add-input is fn (value : Int) -> Int
    value + input
  add-input 2
answer 40

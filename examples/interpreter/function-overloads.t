#!/usr/bin/env topal
# Demonstrates a same-name overload set selecting its Int and String headers
# independently, in source declaration order and without using result type.
describe is fn (value : Int) -> String
  "integer"
describe is fn (value : String) -> String
  value
(describe 42, describe "Topal")

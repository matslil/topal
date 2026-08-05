#!/usr/bin/env topal
# Demonstrates reversible overload selection with explicit signature reasons.
describe is fn (value : Int) -> String
  "integer"
describe is fn (value : String) -> String
  value
(describe 42, describe "Topal")

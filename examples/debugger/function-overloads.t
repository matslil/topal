#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible overload selection with explicit signature reasons.
describe is fn (value : Int) -> String
  "integer"
describe is fn (value : String) -> String
  value
(describe 42, describe "Topal")

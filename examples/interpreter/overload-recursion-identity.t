#!/usr/bin/env topal
# Demonstrates that a call from the String overload to the distinct Int overload
# is ordinary dispatch, not recursion merely because both share a name.
describe is fn (value : Int) -> String
  "integer"
describe is fn (value : String) -> String
  (describe 42) concat ":" concat value
describe "Topal"

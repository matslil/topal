#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible dispatch between distinct overload identities.
describe is fn (value : Int) -> String
  "integer"
describe is fn (value : String) -> String
  (describe 42) concat ":" concat value
describe "Topal"

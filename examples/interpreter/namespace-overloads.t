#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that a namespace alias retains typed, source-ordered overloads.
identity is fn (value : Int) -> Int
  value
identity is fn (value : String) -> String
  value
api is root
(api identity 42, api identity "Topal")

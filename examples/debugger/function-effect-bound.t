#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible declaration and execution of a function whose
# contract explicitly permits no effects.
identity is fn ( value : Int ) -> Int
  : Effects ()
  value
identity 42

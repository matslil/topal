#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible matching of a checked, non-binding function input.
second is fn (_ : Int, value : Int) -> Int
  value
second (0, 42)

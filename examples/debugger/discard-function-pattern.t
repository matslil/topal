#!/usr/bin/env topal
# Demonstrates reversible matching of a checked, non-binding function input.
second is fn (_ : Int, value : Int) -> Int
  value
second (0, 42)

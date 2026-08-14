#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a discard pattern in a typed function input: the first component
# is checked as Int but does not introduce a binding.
second is fn (_ : Int, value : Int) -> Int
  value
second (0, 42)

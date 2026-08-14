#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates static zero- and one-parameter functions with explicit types.
answer is fn static () -> Int
  40 + 2
increment is fn static (input : Int) -> Int
  input + 1
(answer (), increment 41)

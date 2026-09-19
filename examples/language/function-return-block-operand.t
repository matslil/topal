#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that a return inside the lexical-block operand of another return
# completes the surrounding function before either outer tail is evaluated.
answer is fn (value : Int) -> Int
  return { return value + 1 }
  1000
answer 41

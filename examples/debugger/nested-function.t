#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible nested declaration, lexical capture, entry, and return.
answer is fn (input : Int) -> Int
  add-input is fn (value : Int) -> Int
    value + input
  add-input 2
answer 40

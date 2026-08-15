#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a declared function retained as a first-class value, rebound
# under another name, and called with its original typed declaration identity.
increment is fn (value : Int) -> Int
  value + 1

operation is increment
operation 41

#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates debugger history for explicit defining-context selection.
offset is 40
add-offset is fn (value : Int) -> Int
  value + @ offset
add-offset 2

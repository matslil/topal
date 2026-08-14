#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible execution through an ordinary runtime function.
subtract is fn (left : Int, right : Int) -> Int
  difference is left - right
  return difference
50 subtract 8

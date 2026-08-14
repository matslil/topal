#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates an ordinary runtime function with a positional-product input,
# local binding, and explicit result return.
subtract is fn (left : Int, right : Int) -> Int
  difference is left - right
  return difference
50 subtract 8

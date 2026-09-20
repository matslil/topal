#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates return propagation after selecting unary List collection.

answer is fn (value : Int) -> Int
  abandoned : List Int is collect { return value + 1 }
  1000
answer 41

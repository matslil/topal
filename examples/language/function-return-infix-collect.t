#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates returns after selecting infix Array and String collection.

array-exit is fn (value : Int) -> Int
  abandoned : Array (1, Int) is { return value + 1 } collect Array
  1000
string-exit is fn (value : Int) -> Int
  abandoned : String is { return value + 2 } collect String
  1000
(array-exit 40) + (string-exit -1)

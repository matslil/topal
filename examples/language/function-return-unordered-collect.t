#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates returns after selecting unordered collection operations.

set-exit is fn (value : Int) -> Int
  abandoned : Set Int is collect-set { return value + 1 }
  1000
bag-exit is fn (value : Int) -> Int
  abandoned : Bag Int is collect-bag { return value + 2 }
  1000
(set-exit 40) + (bag-exit -1)

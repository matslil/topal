#!/usr/bin/env topal
use language (version is v0.1)
use library advent-of-code (version is v0.1)
count-fitting is advent-of-code packing fitting-region-count
solve is fn (input : String) -> Int
  count-fitting input

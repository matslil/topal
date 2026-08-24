#!/usr/bin/env topal
use language (version is v0.1)
use library std (version is v0.1)
count-fitting is std packing fitting-region-count
solve is fn (input : String) -> Int
  count-fitting input

#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

parse is std parse integer-pairs
largest-contained is std geometry largest-contained-rectangle

solve is fn (input : String) -> Int
  largest-contained (parse input)

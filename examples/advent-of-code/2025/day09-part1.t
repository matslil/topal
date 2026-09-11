#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)
use library advent-of-code (
  version is v0.1
)

parse is std parse integer-pairs
largest is advent-of-code geometry largest-point-rectangle

solve is fn (input : String) -> Int
  largest (parse input)

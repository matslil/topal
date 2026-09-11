#!/usr/bin/env topal
use language (version is v0.1)
use library advent-of-code (version is v0.1)
count-paths is advent-of-code graph described-path-count
solve is fn (input : String) -> Int
  count-paths (input, ("you", "out"))

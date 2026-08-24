#!/usr/bin/env topal
use language (version is v0.1)
use library std (version is v0.1)
count-paths is std graph described-path-count
solve is fn (input : String) -> Int
  count-paths (input, ("you", "out"))

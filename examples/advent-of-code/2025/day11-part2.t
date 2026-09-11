#!/usr/bin/env topal
use language (version is v0.1)
use library advent-of-code (version is v0.1)
count-paths is advent-of-code graph described-required-path-count
required : List String is Entry ("dac", Entry ("fft", Empty))
solve is fn (input : String) -> Int
  count-paths (input, ("svr", "out", required))

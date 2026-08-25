#!/usr/bin/env topal
use language (version is v0.1)
use library std (version is v0.1)
count-paths is std graph described-required-path-count
required : List String is Entry ("dac", Entry ("fft", Empty))
solve is fn (input : String) -> Int
  count-paths (input, ("svr", "out", required))

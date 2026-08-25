#!/usr/bin/env topal
use language (version is v0.1)
use library std (version is v0.1)
minimum is std machine minimum-indicator-presses
solve is fn (input : String) -> Int
  minimum input

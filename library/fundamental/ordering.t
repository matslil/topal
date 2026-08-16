#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates an ordinary Topal standard-library function over a fundamental
# type. The interpreter executes this source; a compiler will lower the same
# published declaration rather than substituting a separate native definition.
pub minimum is fn (left : Int, right : Int) -> Int
  left
    < right then left
    otherwise right

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

pub maximum is fn (left : Int, right : Int) -> Int
  left
    > right then left
    otherwise right

pub between-inclusive is fn (value : Int, bounds : Range Int) -> Boolean
  value in bounds

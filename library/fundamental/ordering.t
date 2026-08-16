#!/usr/bin/env topal
use language (
  version is v0.1
)
# Select the lesser or greater of two Int values without introducing a sorting
# or collection dependency.
pub minimum is fn (left : Int, right : Int) -> Int
  left
    < right then left
    otherwise right

pub maximum is fn (left : Int, right : Int) -> Int
  left
    > right then left
    otherwise right

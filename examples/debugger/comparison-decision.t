#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible comparison matching and fallback selection.
minimum is fn (left : Int, right : Int) -> Int
  left
    < right then left
    otherwise right
(42 minimum 50, 60 minimum 50)

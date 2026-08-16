#!/usr/bin/env topal
use language (
  version is v0.1
)

# Derived algorithms over exact, arbitrary-precision numbers. Parsing and
# presentation policy intentionally remain outside this module.
pub sign is fn (value : Int) -> Int
  value
    < 0 then -1
    > 0 then 1
    otherwise 0

pub distance is fn (left : Int, right : Int) -> Nat
  absolute (left - right)

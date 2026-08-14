#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates direct recursion using positive literal steps larger than one in
# both directions; inclusive bounds make overshooting safe.
down-hops is fn (value : Int) -> Int
  value
    <= 0 then 0
    otherwise 1 + (down-hops (value - 3))
up-hops is fn (value : Int) -> Int
  value
    >= 0 then 0
    otherwise 1 + (up-hops (value + 2))
(down-hops 7, up-hops (-5))

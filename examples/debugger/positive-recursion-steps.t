#!/usr/bin/env topal
# Demonstrates reversible bounded recursion with non-unit positive steps.
down-hops is fn (value : Int) -> Int
  value
    <= 0 then 0
    otherwise 1 + (down-hops (value - 3))
up-hops is fn (value : Int) -> Int
  value
    >= 0 then 0
    otherwise 1 + (up-hops (value + 2))
(down-hops 7, up-hops (-5))

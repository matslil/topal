#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates returns from every action of an ordered comparison decision.

choose is fn (value : Int) -> Int
  value
    < 0 then { return 40 }
    = 0 then { return 41 }
    otherwise { return 42 }
  1000
(choose (-1), choose 0, choose 1)

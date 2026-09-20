#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates returns from every action of a complete Boolean decision.

choose is fn (value : Boolean) -> Int
  value
    false then { return 40 }
    true then { return 41 }
  1000
(choose false, choose true)

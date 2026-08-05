#!/usr/bin/env topal
# Demonstrates reversible consideration and selection of Boolean decision rules.
choose is fn (condition : Boolean) -> Int
  condition
    true then 42
    otherwise 0
(choose true, choose false)

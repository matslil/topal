#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a complete Boolean decision table. The subject is evaluated once,
# rules are considered in order, and only the selected action is evaluated.
choose is fn (condition : Boolean) -> Int
  condition
    true then 42
    otherwise 0
(choose true, choose false)

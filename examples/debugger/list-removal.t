#!/usr/bin/env topal-debug
use language (
  version is v0.1
)
# Demonstrates reversible value-based removal while retaining immutable source
# order and distinguishing the first equal entry from all equal entries.
values : List Int is Entry ( 1, Entry ( 2, Entry ( 3, Entry ( 2, Entry ( 4, Empty ) ) ) ) )
(values remove-first 2, values remove-all 2, values remove-all 9)

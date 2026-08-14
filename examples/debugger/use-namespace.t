#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible namespace selection with use and later qualified lookup.
increment is fn (value : Int) -> Int
  value + 1

current is use root
current increment 41

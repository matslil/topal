#!/usr/bin/env topal
# Demonstrates reversible namespace selection with use and later qualified lookup.
increment is fn (value : Int) -> Int
  value + 1

current is use root
current increment 41

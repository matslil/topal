#!/usr/bin/env topal
# Demonstrates making a namespace available without flattening its members.
increment is fn (value : Int) -> Int
  value + 1

current is use root
current increment 41

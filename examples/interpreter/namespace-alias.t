#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates an immutable alias of the root namespace. Qualified lookup keeps
# the original namespace boundary before invoking the selected member.
increment is fn (value : Int) -> Int
  value + 1

current is root
current increment 41

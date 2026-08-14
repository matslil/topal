#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible binding and member resolution through a namespace
# alias without flattening the namespace into local bindings.
increment is fn (value : Int) -> Int
  value + 1

current is root
current increment 41

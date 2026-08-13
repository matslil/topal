#!/usr/bin/env topal
# Demonstrates reversible binding and member resolution through a namespace
# alias without flattening the namespace into local bindings.
increment is fn (value : Int) -> Int
  value + 1

current is root
current increment 41

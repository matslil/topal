#!/usr/bin/env topal-debug
use language (
  version is v0.1
)
# Demonstrates reversible inferred anonymous calls while mapping, selecting,
# and folding a List. Captured bindings remain immutable at every checkpoint.
factor : Int is 2
values : List Int is Entry ( 1, Entry ( 2, Entry ( 3, Empty ) ) )
(values map { value } value * factor, values select { value } value > 1, values fold 0 { sum, value } sum + value)

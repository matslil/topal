#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible capture, binding, resolution, and invocation of
# inferred anonymous function values used by collection operations.
values : List Int is Entry (1, Entry (2, Entry (3, Empty)))
double is { value } value * 2
sum is { state, value } state + value
(values map double, values fold 0 sum)

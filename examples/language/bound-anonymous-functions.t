#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates first-class inferred anonymous functions: the transformations
# are bound as values before being supplied to List map, select, and fold.
values : List Int is Entry (1, Entry (2, Entry (3, Empty)))
double is { value } value * 2
positive is { value } value > 0
sum is { state, value } state + value
(values map double, values select positive, values fold 0 sum)

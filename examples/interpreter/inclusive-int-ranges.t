#!/usr/bin/env topal
# Demonstrates inclusive Int ranges and both membership spellings. Reversed
# bounds construct an empty predicate rather than enumerating backward.
interval is 0 .. 10
empty-interval is 10 .. 0
includes-five is fn (candidate : Range Int) -> Boolean
  candidate contains 5
(interval, includes-five interval, interval contains 11, includes-five empty-interval)

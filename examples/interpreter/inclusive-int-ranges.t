#!/usr/bin/env topal
# Demonstrates inclusive Int ranges and both membership spellings. Reversed
# bounds construct an empty predicate rather than enumerating backward.
interval is 0 .. 10
empty-interval is 10 .. 0
preserve is fn (candidate : Range Int) -> Range Int
  candidate
includes-five is fn (candidate : Range Int) -> Boolean
  candidate contains 5
(preserve interval, interval and (5 .. 15), interval and (20 .. 30), includes-five interval, interval contains 11, includes-five empty-interval)

#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates the four endpoint forms and both membership spellings. Reversed
# bounds construct an empty predicate rather than enumerating backward.
interval is 0 ..= 10
half-open is 0 .. 10
open is 0 <.. 10
lower-open is 0 <..= 10
empty-interval is 10 ..= 0
preserve is fn (candidate : Range Int) -> Range Int
  candidate
includes-five is fn (candidate : Range Int) -> Boolean
  candidate contains 5
(preserve interval, half-open, open, lower-open, 0 in half-open, half-open contains 10, 0 in open, open contains 10, 0 in lower-open, lower-open contains 10, interval and (5 ..= 15), interval and (20 ..= 30), includes-five interval, interval contains 11, includes-five empty-interval)

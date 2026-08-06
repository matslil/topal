#!/usr/bin/env topal
# Demonstrates reversible range construction and membership decisions.
interval is 0 .. 10
empty-interval is 10 .. 0
includes-five is fn (candidate : Range Int) -> Boolean
  candidate contains 5
(interval, includes-five interval, interval contains 11, includes-five empty-interval)

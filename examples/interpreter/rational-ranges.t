#!/usr/bin/env topal
# Demonstrates exact Rational ranges, including canonical Int conversion for a
# mixed endpoint and for membership without rounding.
interval is 0 .. 2.5
includes-one is fn (candidate : Range Rational) -> Boolean
  candidate contains 1
(interval, 1.5 in interval, includes-one interval, 3 in interval)

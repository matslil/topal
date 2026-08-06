#!/usr/bin/env topal
# Demonstrates reversible Rational range construction and exact membership.
interval is 0 .. 2.5
includes-one is fn (candidate : Range Rational) -> Boolean
  candidate contains 1
(interval, 1.5 in interval, includes-one interval, 3 in interval)

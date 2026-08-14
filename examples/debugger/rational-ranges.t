#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible Rational range construction and exact membership.
interval is 0 .. 2.5
preserve is fn (candidate : Range Rational) -> Range Rational
  candidate
includes-one is fn (candidate : Range Rational) -> Boolean
  candidate contains 1
(preserve interval, interval and (1.0 .. 3.0), 1.5 in interval, includes-one interval, 3 in interval)

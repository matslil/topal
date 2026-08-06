#!/usr/bin/env topal
# Demonstrates exact Rational ranges, including canonical Int conversion for a
# mixed endpoint and for membership without rounding.
interval is 0 .. 2.5
(interval, 1.5 in interval, interval contains 2, 3 in interval)

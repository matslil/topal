#!/usr/bin/env topal
# Demonstrates reversible range construction and membership decisions.
interval is 0 .. 10
empty-interval is 10 .. 0
(interval, 5 in interval, interval contains 11, 5 in empty-interval)

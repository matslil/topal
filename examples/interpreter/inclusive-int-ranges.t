#!/usr/bin/env topal
# Demonstrates inclusive Int ranges and both membership spellings. Reversed
# bounds construct an empty predicate rather than enumerating backward.
interval is 0 .. 10
empty-interval is 10 .. 0
(interval, 5 in interval, interval contains 11, 5 in empty-interval)

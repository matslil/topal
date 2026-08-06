#!/usr/bin/env topal
# Demonstrates inclusive Int ranges. Reversed bounds construct an empty range;
# ranges are predicates and do not imply that their members are enumerated.
(0 .. 10, 5 .. 5, 10 .. 0)

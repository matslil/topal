#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible overload selection for exact numeric absolute value.
(absolute -42, absolute 42, absolute -1.25, absolute 1.25)

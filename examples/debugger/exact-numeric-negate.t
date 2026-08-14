#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible named negation overload selection for exact numbers.
(negate 42, negate -42, negate 1.25, negate -1.25)

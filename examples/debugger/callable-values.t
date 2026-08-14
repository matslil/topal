#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible capture, binding, and direct application of symbolic
# callable values for binary addition and unary negation.
add is +
negate is -
(add (20, 22), negate 5)

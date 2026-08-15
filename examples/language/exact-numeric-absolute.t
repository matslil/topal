#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact absolute value for Int and Rational; each result retains
# the input numeric domain and requires no failure path.
(absolute -42, absolute 42, absolute -1.25, absolute 1.25)

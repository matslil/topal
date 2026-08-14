#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates named exact negation for Int and Rational. It returns the same
# values as prefix negation through an ordinary named-application path.
(negate 42, negate -42, negate 1.25, negate -1.25)

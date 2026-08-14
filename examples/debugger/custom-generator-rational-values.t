#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible preservation of an exact Rational while a generator
# is suspended and resumed before its distinct final return.
next is generator ( initial : Rational )
  yields Rational
  resumes Unit
  -> Rational

  _ is yield initial
  initial + (Rational (1, 3))

generated is next (Rational (1, 3))
generated foreach { value }
  _ is value + (Rational (1, 3))

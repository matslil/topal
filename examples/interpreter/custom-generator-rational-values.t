#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact Rational values across generator input, yield, suspension,
# Unit resumption, and a distinct final return without finite conversion.
next is generator ( initial : Rational )
  yields Rational
  resumes Unit
  -> Rational

  _ is yield initial
  initial + (Rational (1, 3))

generated is next (Rational (1, 3))
generated foreach { value }
  _ is value + (Rational (1, 3))

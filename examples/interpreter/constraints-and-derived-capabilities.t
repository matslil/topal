#!/usr/bin/env topal
# Demonstrates a named value constraint, successful evidence construction,
# evidence-forgetting conversion back to Int, and equality/ordering derived from
# the constrained base type. The commented failing form is a static diagnostic.
Positive is Int constraint { value } value > 0
first : Positive is Positive 3
second : Positive is Positive 5
validate is fn (value : Int) -> Result (Int, lang arithmetic ArithmeticErrorCode)
  Positive value

(first, first = (Positive 3), first < second, first + 2, validate 0)
# Positive 0  # E-CONSTRAINT-REJECTED: the closed value violates Positive.

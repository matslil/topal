#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact Rational construction: Int embeds with denominator one,
# common factors are removed, denominator sign moves, and zero is canonical.
(Rational 7, Rational (2, 4), Rational (2, -4), Rational (0, 5))

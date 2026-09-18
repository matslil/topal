#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates contextual Rational infinities and explicit infinite Rational endpoints.
negative : Rational is -Infinity
positive : Rational is +Infinity
positive-again : Rational is +Infinity
finite : Rational is Rational (3, 2)
zero : Rational is Rational (0, 1)
whole is negative ..= positive
nonnegative is zero ..= positive
window is (Rational (-2, 1)) .. (Rational (2, 1))
clipped is whole and window
mixed is (-1) ..= positive
(
  negative,
  positive,
  negative < finite,
  positive > 10,
  negative <=> positive,
  positive = positive-again,
  range-lower whole,
  range-upper whole,
  negative in whole,
  positive in whole,
  empty? (positive .. negative),
  whole,
  range-lower nonnegative,
  negative in nonnegative,
  nonnegative,
  positive in window,
  clipped,
  empty? clipped,
  (Rational (1, 1)) in mixed,
  mixed
)

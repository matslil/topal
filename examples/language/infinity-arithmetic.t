#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates total exact infinity arithmetic and preserved numeric domains.
positive : Int is +Infinity
negative : Int is -Infinity
natural : Nat is +Infinity
rational-positive : Rational is +Infinity
rational-negative : Rational is -Infinity
integerNegative is positive * (-3)
integerPositive is negative * negative
rationalPositiveResult is rational-negative * (Rational (-2, 3))
rationalNegativeResult is rational-positive * rational-negative
(
  positive + 42,
  -10 + negative,
  positive + positive,
  positive - (-7),
  7 - positive,
  positive - negative,
  integerNegative,
  integerPositive,
  negate positive,
  absolute negative,
  natural + 2,
  natural * 4,
  rational-positive + (Rational (-1, 2)),
  3 + rational-negative,
  (Rational (1, 3)) - rational-positive,
  rationalPositiveResult,
  rationalNegativeResult,
  negate rational-positive,
  absolute rational-negative
)

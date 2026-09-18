#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates contextual exact infinities and explicit infinite range endpoints.
negative : Int is -Infinity
positive : Int is +Infinity
natural : Nat is +Infinity
whole is negative ..= positive
nonnegative is whole and (0 ..= positive)
finite is 0 ..= 10
clipped is finite and whole
(
  negative,
  positive,
  natural,
  negative < 0,
  0 < positive,
  negative <=> positive,
  positive = natural,
  range-lower whole,
  range-upper whole,
  negative in whole,
  positive in whole,
  empty? (positive .. negative),
  whole,
  range-lower nonnegative,
  negative in nonnegative,
  nonnegative,
  positive in finite,
  clipped,
  empty? clipped,
  10 in clipped
)

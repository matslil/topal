#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact infinity passage through executable-private, non-recursive
# function parameters/results, captured environments, and Tuple/Record
# aggregates.
identity-int is fn (value : Int) -> Int
  value

identity-nat is fn (value : Nat) -> Nat
  value

identity-rational is fn (value : Rational) -> Rational
  value

offset-int is fn (value : Int) -> Int
  value + 2

return-pair is fn (value : (Int, Rational)) -> (Int, Rational)
  value

return-record is fn (value : Record (integer : Int, ratio : Rational)) -> Record (integer : Int, ratio : Rational)
  value

make-negative is fn () -> Int
  -Infinity

capture-positive is fn () -> Int
  root positive

positive : Int is +Infinity
natural : Nat is +Infinity
rational-negative : Rational is -Infinity
integer-result is identity-int positive
natural-result is identity-nat natural
rational-result is identity-rational rational-negative
offset-result is offset-int positive
pair-result is return-pair (positive, rational-negative)
record-result is return-record (integer is positive, ratio is rational-negative)
constructed-result is make-negative ()
captured-result is capture-positive ()
widened-natural-result is identity-int natural
(
  integer-result,
  natural-result,
  rational-result,
  offset-result,
  pair-result,
  record-result,
  constructed-result,
  captured-result,
  widened-natural-result
)

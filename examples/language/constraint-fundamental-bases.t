#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates constraints over every admitted fundamental scalar base.

Pass is Boolean constraint { value } value
Nonempty is String constraint { value } value != ""
PositiveRational is Rational constraint { value } value > 0
Small is Nat constraint { value } value < 10

accepted : Pass is Pass true
name : Nonempty is Nonempty "Topal"
ratio : PositiveRational is PositiveRational (Rational (3, 2))

validate-pass is fn (value : Boolean) -> Result (Boolean, lang arithmetic ArithmeticErrorCode)
  Pass value

validate-name is fn (value : String) -> Result (String, lang arithmetic ArithmeticErrorCode)
  Nonempty value

validate-ratio is fn (value : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  PositiveRational value

validate-small is fn (value : Nat) -> Result (Nat, lang arithmetic ArithmeticErrorCode)
  Small value

(accepted, name, ratio, validate-pass true, validate-pass false, validate-name "native", validate-name "", validate-ratio (Rational (1, 2)), validate-ratio (Rational (-1, 2)), validate-small 9, validate-small 10)

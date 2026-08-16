#!/usr/bin/env topal
use language (
  version is v0.1
)

# Demonstrates total Optional and Result decisions. Fallback values are
# explicit; Result recovery preserves the original Error unless the caller
# deliberately selects a replacement value.
pub optional-int-or is fn (candidate : Optional Int, fallback : Int) -> Int
  candidate
    Some value then value
    None then fallback

pub result-rational-or is fn (
  candidate : Result (Rational, lang arithmetic ArithmeticErrorCode),
  fallback : Rational
) -> Rational
  candidate
    Ok value then value
    Error problem then fallback

pub result-rational-failed is fn (
  candidate : Result (Rational, lang arithmetic ArithmeticErrorCode)
) -> Boolean
  candidate
    Ok value then false
    Error problem then true

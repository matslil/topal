#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible execution of a successful explicit Result contract.
identity is fn (value : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  value
identity 1.5

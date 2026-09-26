#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates returns from both actions of an exhaustive Result decision.

divide is fn (candidate : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  42.0 / candidate

choose is fn (candidate : Rational) -> Boolean
  divide candidate
    Ok quotient then { return quotient = quotient }
    Error problem then { return (problem code) = (problem code) }
  1000

(choose 1.0, choose 0.0)

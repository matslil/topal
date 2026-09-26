#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates returns from complete qualified Error-code decision actions.

ratio is fn (numerator : Int, denominator : Int) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  Rational (numerator, denominator)

recover is fn (numerator : Int, denominator : Int) -> Boolean
  ratio (numerator, denominator)
    Ok quotient then { return quotient = quotient }
    Error ( code is lang arithmetic division-by-zero ) then { return false }
    Error problem then { return (problem code) = (problem code) }
  1000

classify is fn (numerator : Int, denominator : Int) -> Int
  ratio (numerator, denominator)
    Ok quotient then { return 0 }
    Error ( code is lang arithmetic out-of-range ) then { return 1 }
    Error ( code is lang arithmetic not-representable ) then { return 2 }
    Error ( code is lang arithmetic division-by-zero ) then { return 3 }
    Error ( code is lang arithmetic indeterminate ) then { return 4 }
  true

(recover (1, 2), recover (1, 0), recover (0, 0), classify (1, 2), classify (1, 0), classify (0, 0))

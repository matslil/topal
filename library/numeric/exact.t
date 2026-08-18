#!/usr/bin/env topal
use language (
  version is v0.1
)

# Derived algorithms over exact, arbitrary-precision numbers. Parsing and
# presentation policy intentionally remain outside this module.
pub sign is fn (value : Int) -> Int
  value
    < 0 then -1
    > 0 then 1
    otherwise 0

pub sign is fn (value : Rational) -> Int
  value
    < 0 then -1
    > 0 then 1
    otherwise 0

pub distance is fn (left : Int, right : Int) -> Nat
  absolute (left - right)

pub distance is fn (left : Rational, right : Rational) -> Rational
  absolute (left - right)

pub gcd is fn (left : Int, right : Int) -> Nat : Decreases (absolute right)
  right
    = 0 then absolute left
    otherwise gcd (right, left % right)

pub even? is fn (value : Int) -> Boolean
  value % 2 = 0

pub odd? is fn (value : Int) -> Boolean
  value % 2 != 0

pub divides? is fn (divisor : Int, dividend : Int) -> Boolean
  divisor
    = 0 then dividend = 0
    otherwise dividend % divisor = 0

pub reciprocal is fn (
  value : Rational
) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  1.0 / value

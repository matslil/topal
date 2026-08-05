#!/usr/bin/env topal
# Demonstrates the successful path of an explicit arithmetic Result contract;
# the Rational value is not wrapped and no Error domain is created.
identity is fn (value : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  value
identity 1.5

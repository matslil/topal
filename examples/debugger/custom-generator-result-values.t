#!/usr/bin/env topal
# Demonstrates reversible Result flow from successful suspension to a structured
# division-by-zero Error retaining its domain, code, and source position.
attempt is generator ( initial : Result (Rational, lang arithmetic ArithmeticErrorCode) )
  yields Result (Rational, lang arithmetic ArithmeticErrorCode)
  resumes Unit
  -> Result (Rational, lang arithmetic ArithmeticErrorCode)

  _ is yield initial
  initial / (Rational 0)

generated is attempt (Rational 1)
generated foreach { candidate }
  _ is candidate = candidate

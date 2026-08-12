#!/usr/bin/env topal
# Demonstrates a successful arithmetic Result crossing input and yield, followed
# by a structured division-by-zero Error as the generator's final Result.
attempt is generator ( initial : Result (Rational, lang arithmetic ArithmeticErrorCode) )
  yields Result (Rational, lang arithmetic ArithmeticErrorCode)
  resumes Unit
  -> Result (Rational, lang arithmetic ArithmeticErrorCode)

  _ is yield initial
  initial / (Rational 0)

generated is attempt (Rational 1)
generated foreach { candidate }
  _ is candidate = candidate

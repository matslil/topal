#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible recursive classifier state for declared Choice values
# nested inside Optional and arithmetic Result generator boundaries.
Choice is Enum ( First, Second )
both is generator ( initial : (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode)) )
  yields (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))
  resumes Unit
  -> (Optional Choice, Result (Choice, lang arithmetic ArithmeticErrorCode))

  _ is yield initial
  (Some Second, Second)

generated is both (Some First, First)
generated foreach { candidate }
  _ is candidate = (Some First, First)

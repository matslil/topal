#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible recursive Result state with an exact positional
# product in the success channel across suspension and final return.
pair is generator ( initial : Result ((Int, String), lang arithmetic ArithmeticErrorCode) )
  yields Result ((Int, String), lang arithmetic ArithmeticErrorCode)
  resumes Unit
  -> Result ((Int, String), lang arithmetic ArithmeticErrorCode)

  _ is yield initial
  (8, "done")

generated is pair (7, "item")
generated foreach { candidate }
  _ is candidate = (7, "item")

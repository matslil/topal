#!/usr/bin/env topal
# Demonstrates recursive Result classification: the successful value is a
# positional (Int, String) product preserved through suspension and return.
pair is generator ( initial : Result ((Int, String), lang arithmetic ArithmeticErrorCode) )
  yields Result ((Int, String), lang arithmetic ArithmeticErrorCode)
  resumes Unit
  -> Result ((Int, String), lang arithmetic ArithmeticErrorCode)

  _ is yield initial
  (8, "done")

generated is pair (7, "item")
generated foreach { candidate }
  _ is candidate = (7, "item")

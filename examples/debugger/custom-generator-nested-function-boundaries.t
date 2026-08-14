#!/usr/bin/env topal-debug
use language (
  version is v0.1
)
# Demonstrates reversible execution across ordinary function boundaries whose
# generator classifier recursively contains Optional, product, and Result.
pairs is generator ( initial : Optional (Int, String) )
  yields Optional (Int, String)
  resumes Unit
  -> Result ((Int, String), lang arithmetic ArithmeticErrorCode)

  _ is yield initial
  (8, "done")

make is fn ( initial : Optional (Int, String) ) -> Generator Optional (Int, String) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode)
  pairs initial

consume is fn ( generated : Generator Optional (Int, String) Unit Result ((Int, String), lang arithmetic ArithmeticErrorCode) ) -> Result ((Int, String), lang arithmetic ArithmeticErrorCode)
  result : Result ((Int, String), lang arithmetic ArithmeticErrorCode) is generated foreach { value }
    _ is value = (Some (7, "item"))
  result

generated is make (Some (7, "item"))
consume generated

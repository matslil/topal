#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates represented List, Optional, Result, and Range values as complete
# package fields with source-order calls and a closed container default.
make-values is fn () -> List Int
  Entry (20, Entry (22, Empty))

make-maybe is fn () -> Optional Int
  Some 42

make-outcome is fn () -> Result (Int, lang arithmetic ArithmeticErrorCode)
  42

make-span is fn () -> Range Int
  40 ..= 42

retain is fn ((values : List Int, maybe : Optional Int default None Int, outcome : Result (Int, lang arithmetic ArithmeticErrorCode), span : Range Int)) -> (List Int, Optional Int, Result (Int, lang arithmetic ArithmeticErrorCode), Range Int)
  (values, maybe, outcome, span)

(
  retain (span is make-span (), outcome is make-outcome (), maybe is make-maybe (), values is make-values ()),
  retain (span is 40 ..= 42, outcome is make-outcome (), values is make-values ())
)

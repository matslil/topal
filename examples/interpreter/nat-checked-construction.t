#!/usr/bin/env topal
# Demonstrates checked Nat constraint construction: nonnegative Int values are
# preserved, while a dynamically obtained negative Int returns out-of-range.
as-nat is fn (value : Int) -> Result (Nat, lang arithmetic ArithmeticErrorCode)
  Nat value
(Nat 7, as-nat 6, as-nat -1)

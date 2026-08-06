#!/usr/bin/env topal
# Demonstrates reversible checked Nat success and dynamic constraint failure.
as-nat is fn (value : Int) -> Result (Nat, lang arithmetic ArithmeticErrorCode)
  Nat value
(Nat 7, as-nat 6, as-nat -1)

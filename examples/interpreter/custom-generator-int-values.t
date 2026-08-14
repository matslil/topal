#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that arbitrary-precision Int values retain their exact value
# across generator input, yield, suspension, resumption, and final return.
next is generator ( initial : Int )
  yields Int
  resumes Unit
  -> Int

  _ is yield initial
  initial + 1

generated is next 999999999999999999999999999999
generated foreach { value }
  _ is value + 1

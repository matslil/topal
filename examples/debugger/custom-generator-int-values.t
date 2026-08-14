#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible arbitrary-precision Int flow through a suspended
# generator and its distinct final return.
next is generator ( initial : Int )
  yields Int
  resumes Unit
  -> Int

  _ is yield initial
  initial + 1

generated is next 999999999999999999999999999999
generated foreach { value }
  _ is value + 1

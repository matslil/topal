#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible transfer of a constrained Nat value through a
# suspended generator and its final return.
next is generator ( initial : Nat )
  yields Nat
  resumes Unit
  -> Nat

  _ is yield initial
  initial + 1

generated is next (Nat 7)
generated foreach { value }
  _ is value + 1

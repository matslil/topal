#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates preservation of positional product component order and types
# across generator input, yield, suspension, and a distinct final product.
pair is generator ( initial : (Int, String) )
  yields (Int, String)
  resumes Unit
  -> (Int, String)

  _ is yield initial
  (8, "done")

generated is pair (7, "item")
generated foreach { value }
  _ is value = (7, "item")

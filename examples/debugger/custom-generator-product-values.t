#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible positional-product flow through a suspended generator,
# preserving component order and classifiers.
pair is generator ( initial : (Int, String) )
  yields (Int, String)
  resumes Unit
  -> (Int, String)

  _ is yield initial
  (8, "done")

generated is pair (7, "item")
generated foreach { value }
  _ is value = (7, "item")

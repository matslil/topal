#!/usr/bin/env topal
# Demonstrates reversible recursive classifier state for an Optional containing
# a positional (Int, String) product across suspension and final return.
pair is generator ( initial : Optional (Int, String) )
  yields Optional (Int, String)
  resumes Unit
  -> Optional (Int, String)

  _ is yield initial
  Some (8, "done")

generated is pair (Some (7, "item"))
generated foreach { candidate }
  _ is candidate = (Some (7, "item"))

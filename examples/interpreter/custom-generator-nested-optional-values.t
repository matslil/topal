#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates recursive generator classifiers: Optional preserves its Some
# alternative while its positional (Int, String) payload retains component order.
pair is generator ( initial : Optional (Int, String) )
  yields Optional (Int, String)
  resumes Unit
  -> Optional (Int, String)

  _ is yield initial
  Some (8, "done")

generated is pair (Some (7, "item"))
generated foreach { candidate }
  _ is candidate = (Some (7, "item"))

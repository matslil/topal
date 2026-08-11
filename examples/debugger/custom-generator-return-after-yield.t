#!/usr/bin/env topal
# Demonstrates reversible tracing of a yield, Unit resumption, and the explicit
# String return that completes the generator continuation.
finish is generator ( initial : String )
  yields String
  resumes Unit
  -> String

  _ is yield initial
  return "done"

generated is finish "item"
generated foreach { text }
  _ is empty? text

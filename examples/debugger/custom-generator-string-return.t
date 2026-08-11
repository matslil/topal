#!/usr/bin/env topal
# Demonstrates reversible String yield, Unit resumption, and distinct final
# String return from one custom continuation.
text-result is generator ( initial : String )
  yields String
  resumes Unit
  -> String

  _ is yield initial
  "done"

generated is text-result "item"
generated foreach { text }
  _ is empty? text

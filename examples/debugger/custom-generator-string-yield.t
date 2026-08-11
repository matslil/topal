#!/usr/bin/env topal
# Demonstrates reversible String yields and Unit resumptions through direct
# foreach traversal of a custom continuation.
texts is generator ( initial : String )
  yields String
  resumes Unit
  -> Unit

  _ is yield initial
  _ is yield ""
  ()

generated is texts "Topal"
generated foreach { text }
  _ is empty? text

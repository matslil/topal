#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible ordering of Unit resumption, a discarded String
# computation, and the next custom-generator suspension.
inspect-between is generator ( initial : String )
  yields String
  resumes Unit
  -> Unit

  _ is yield initial
  _ is empty? initial
  _ is yield ""
  ()

generated is inspect-between "Topal"
generated foreach { text }
  _ is empty? text

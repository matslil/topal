#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a String yield followed by a distinct final String. Foreach
# consumes `item`, resumes with Unit, and then produces `done`.
text-result is generator ( initial : String )
  yields String
  resumes Unit
  -> String

  _ is yield initial
  "done"

generated is text-result "item"
generated foreach { text }
  _ is empty? text

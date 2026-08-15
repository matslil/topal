#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates an ordinary discarded computation between yields. It executes
# only after foreach resumes the first yield and before the second suspension.
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

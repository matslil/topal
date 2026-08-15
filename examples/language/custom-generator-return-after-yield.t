#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates explicit return after a yielded String. Foreach invokes the action,
# resumes the generator with Unit, and then produces its declared final String.
finish is generator ( initial : String )
  yields String
  resumes Unit
  -> String

  _ is yield initial
  return "done"

generated is finish "item"
generated foreach { text }
  _ is empty? text

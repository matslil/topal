#!/usr/bin/env topal
# Demonstrates that initial input is independent of yield/resume/return types.
# A String predicate runs before the continuation yields one Character.
from-text is generator ( initial : String )
  yields Character
  resumes Unit
  -> Unit

  initial-is-empty : Boolean is empty? initial
  _ is yield "T"
  ()

generated is from-text "Topal"
generated foreach { character }
  _ is String character

#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible String initial-input evaluation before the custom
# continuation suspends with a Character yield.
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

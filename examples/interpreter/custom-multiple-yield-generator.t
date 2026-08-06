#!/usr/bin/env topal
# Demonstrates that a custom generator can yield repeatedly in source order.
# Direct foreach resumes it with Unit after each Character and returns Unit.
twice is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  _ is yield initial
  _ is yield initial
  ()

generated is twice "T"
generated foreach { character }
  _ is String character

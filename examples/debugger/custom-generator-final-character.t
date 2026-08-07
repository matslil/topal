#!/usr/bin/env topal
# Demonstrates reversible observation of a yielded Character followed by a
# distinct final Character return from the same generator continuation.
yield-then-return is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Character

  _ is yield initial
  "R"

generated is yield-then-return "Y"
generated foreach { character }
  _ is String character

#!/usr/bin/env topal
# Demonstrates reversible suspension and resumption, including a local binding
# that cannot execute until after the first Unit resumption.
pause-twice is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  _ is yield initial
  copy : Character is initial
  _ is yield copy
  ()

generated is pause-twice "T"
generated foreach { character }
  _ is String character

#!/usr/bin/env topal
# Demonstrates true suspension. The local `copy` binding after the first yield
# is created only when foreach resumes with Unit, before the second yield.
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

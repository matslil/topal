#!/usr/bin/env topal
# Demonstrates reversible creation of a resume-result binding after foreach
# supplies Unit to the suspended yield expression.
bind-resume is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  resumed is yield initial
  resumed

generated is bind-resume "T"
generated foreach { character }
  _ is String character

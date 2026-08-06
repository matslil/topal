#!/usr/bin/env topal
# Demonstrates reversible creation and use of a generator-local binding before
# the continuation yields its Character and later returns Unit.
copy-once is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  copy : Character is initial
  _ is yield copy
  ()

generated is copy-once "T"
generated foreach { character }
  _ is String character

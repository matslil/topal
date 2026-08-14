#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible repeated yields and Unit resumptions from one custom
# generator continuation, followed by its final Unit return.
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

#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates applying a generator selected through a namespace alias.
once is generator (initial : Character)
  yields Character
  resumes Unit
  -> Unit
  _ is yield initial
  ()
api is root
generated is api once "T"
generated foreach { character }
  _ is String character

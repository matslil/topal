#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates generator-local state. The `copy` binding is created inside the
# generator and is available to its later yield, but never leaks to the caller.
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
